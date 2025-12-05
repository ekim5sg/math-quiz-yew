export interface Env {
  OPENAI_API_KEY: string; // set via wrangler secret
}

type Difficulty = "easy" | "moderate" | "advanced";
type Op = "add" | "sub" | "mul" | "div";

function clamp(n: number, min: number, max: number) {
  return Math.min(Math.max(n, min), max);
}

function randInt(min: number, max: number) {
  return Math.floor(Math.random() * (max - min + 1)) + min;
}

function pickOp(d: Difficulty): Op {
  // tilt operations by difficulty
  const easyOps: Op[] = ["add", "sub"];
  const modOps: Op[] = ["add", "sub", "mul"];
  const advOps: Op[] = ["add", "sub", "mul", "div"];
  const pool = d === "easy" ? easyOps : d === "moderate" ? modOps : advOps;
  return pool[randInt(0, pool.length - 1)];
}

function buildOperands(op: Op, difficulty: Difficulty, maxN: number) {
  // keep things friendly to 2nd/3rd grade ranges
  const maxCap = difficulty === "easy" ? Math.min(maxN, 20)
                : difficulty === "moderate" ? Math.min(maxN, 50)
                : Math.min(maxN, 99);

  switch (op) {
    case "add": {
      const a = randInt(0, maxCap);
      const b = randInt(0, maxCap - a > 0 ? maxCap - a : maxCap); // sometimes smaller sum
      return { a, b, answer: a + b };
    }
    case "sub": {
      const a = randInt(0, maxCap);
      const b = randInt(0, a); // non-negative result
      return { a, b, answer: a - b };
    }
    case "mul": {
      // friendly tables by difficulty
      const hi = difficulty === "easy" ? 5 : difficulty === "moderate" ? 10 : 12;
      const a = randInt(0, hi);
      const b = randInt(0, hi);
      return { a, b, answer: a * b };
    }
    case "div": {
      // whole-number division only
      const hi = difficulty === "easy" ? 5 : difficulty === "moderate" ? 10 : 12;
      const divisor = randInt(1, hi);
      const quotient = randInt(1, hi);
      const dividend = divisor * quotient;
      return { a: dividend, b: divisor, answer: quotient };
    }
  }
}

// Small, direct call to OpenAI to *phrase* the problem.
// We pass operation + operands and keep the model from inventing numbers/answers.
async function makeWordProblem(env: Env, op: Op, a: number, b: number, difficulty: Difficulty): Promise<string> {
  const operationWord =
    op === "add" ? "addition" :
    op === "sub" ? "subtraction" :
    op === "mul" ? "multiplication" : "division";

  const system = `You write very short, kid-friendly WORD PROBLEMS for 2nd–3rd graders.
Use only the numbers and operation provided. 1–2 short sentences.
No equations in the text; no variables; no extra numbers. One clear final question.`;

  const user = `Create a ${operationWord} word problem using ONLY these numbers: ${a} and ${b}.
- Difficulty: ${difficulty}
- Requirements:
  - Use the numbers exactly as provided (no new numbers).
  - Make the story wholesome and concrete (stickers, apples, books, coins, marbles, etc.).
  - End with a single question that implies a whole-number answer.
  - Do NOT include the equation or the answer.
Examples of tone: "Kiki has 7 stickers and gets 5 more. How many does she have now?"`;

  const resp = await fetch("https://api.openai.com/v1/chat/completions", {
    method: "POST",
    headers: {
      "Authorization": `Bearer ${env.OPENAI_API_KEY}`,
      "Content-Type": "application/json",
    },
    body: JSON.stringify({
      model: "gpt-4o-mini",
      temperature: 0.4,
      max_tokens: 80,
      messages: [
        { role: "system", content: system },
        { role: "user", content: user }
      ],
    }),
  });

  if (!resp.ok) {
    throw new Error(`OpenAI error ${resp.status}`);
  }
  const data = await resp.json();
  const text: string =
    data?.choices?.[0]?.message?.content?.trim?.() ??
    "A student-friendly word problem could not be generated.";

  return text;
}

function okHeaders() {
  return {
    "Content-Type": "application/json",
    "Access-Control-Allow-Origin": "*",           // consider restricting to your domain
    "Access-Control-Allow-Methods": "POST,OPTIONS",
    "Access-Control-Allow-Headers": "Content-Type",
  };
}

export default {
  async fetch(request: Request, env: Env): Promise<Response> {
    if (request.method === "OPTIONS") {
      return new Response(null, { headers: okHeaders() });
    }
    if (request.method !== "POST") {
      return new Response(JSON.stringify({ error: "Use POST with JSON body." }), {
        status: 405, headers: okHeaders()
      });
    }

    let body: any;
    try {
      body = await request.json();
    } catch {
      return new Response(JSON.stringify({ error: "Invalid JSON." }), {
        status: 400, headers: okHeaders()
      });
    }

    const difficulty = (body?.difficulty ?? "easy").toString().toLowerCase() as Difficulty;
    const maxRaw = Number(body?.max_number ?? 20);

    if (!["easy", "moderate", "advanced"].includes(difficulty)) {
      return new Response(JSON.stringify({ error: "difficulty must be 'easy' | 'moderate' | 'advanced'." }), {
        status: 400, headers: okHeaders()
      });
    }
    const max_number = clamp(isFinite(maxRaw) ? maxRaw : 20, 10, 200); // guardrails

    try {
      // choose op + operands and compute answer *server-side*
      const op = pickOp(difficulty);
      const { a, b, answer } = buildOperands(op, difficulty, max_number);

      // ask OpenAI to phrase the story, but *not* the answer
      const prompt = await makeWordProblem(env, op, a, b, difficulty);

      return new Response(JSON.stringify({ prompt, answer }), {
        headers: okHeaders()
      });
    } catch (err: any) {
      // soft failure path: minimal local fallback prompt (still returns correct answer)
      const op: Op = "add";
      const { a, b, answer } = buildOperands(op, "easy", 20);
      const fallback = `Kiki has ${a} stickers and gets ${b} more. How many stickers does she have now?`;

      return new Response(JSON.stringify({
        prompt: fallback,
        answer,
        note: "AI unavailable; returned local fallback."
      }), { status: 200, headers: okHeaders() });
    }
  }
} satisfies ExportedHandler<Env>;
#![allow(warnings)]

use js_sys::Math;
use web_sys::{HtmlInputElement, console};
use yew::prelude::*;
use yew::TargetCast;

use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use wasm_bindgen_futures::spawn_local;

// Tiny helper to log to browser console
fn log(msg: &str) {
    console::log_1(&msg.into());
}

// Your deployed Worker URL
const AI_WORKER_URL: &str = "https://math-quiz-word-worker.mikegyver.workers.dev/";

#[derive(Clone, PartialEq)]
enum Difficulty {
    Easy,
    Moderate,
    Advanced,
}

#[derive(Clone, PartialEq)]
struct QuizConfig {
    num_questions: usize,
    difficulty: Difficulty,
    include_add: bool,
    include_sub: bool,
    include_mul: bool,
    include_div: bool,
    include_words: bool,
}

#[derive(Clone, PartialEq)]
struct Question {
    prompt: String,
    kind: String,
    answer: i32,
    user_answer: String,
    is_correct: Option<bool>,
}

fn default_config() -> QuizConfig {
    QuizConfig {
        num_questions: 10,
        difficulty: Difficulty::Easy,
        include_add: true,
        include_sub: true,
        include_mul: false,
        include_div: false,
        include_words: true,
    }
}

/// Payload sent to your Cloudflare Worker
#[derive(Serialize)]
struct AiWordProblemRequest {
    difficulty: String,
    max_number: i32,
}

/// Response expected back from the Worker
#[derive(Deserialize)]
struct AiWordProblemResponse {
    prompt: String,
    answer: i32,
}

/// Random integer in [min, max], inclusive
fn rand_int(min: i32, max: i32) -> i32 {
    let r = Math::random();
    min + ((r * ((max - min + 1) as f64)) as i32)
}

#[derive(Clone, Copy)]
enum BaseOp {
    Add,
    Sub,
    Mul,
    Div,
}

fn difficulty_code(diff: &Difficulty) -> &'static str {
    match diff {
        Difficulty::Easy => "easy",
        Difficulty::Moderate => "moderate",
        Difficulty::Advanced => "advanced",
    }
}

fn difficulty_label(diff: &Difficulty) -> &'static str {
    match diff {
        Difficulty::Easy => "Easy",
        Difficulty::Moderate => "Moderate",
        Difficulty::Advanced => "Advanced",
    }
}

/// Return (question text, answer, kind label)
/// Now with clearer difficulty tiers:
/// - Easy: single-digit for + / −, small × / ÷
/// - Moderate: two-digit for + / −, bigger × / ÷
/// - Advanced: three-digit for + / −, beefy × / ÷
fn generate_basic_question(cfg: &QuizConfig, op: BaseOp) -> (String, i32, String) {
    match op {
        BaseOp::Add => {
            let (min, max) = match cfg.difficulty {
                Difficulty::Easy => (0, 9),        // single-digit
                Difficulty::Moderate => (10, 99),  // two-digit
                Difficulty::Advanced => (100, 999),// three-digit
            };
            let a = rand_int(min, max);
            let b = rand_int(min, max);
            (format!("{a} + {b} = ?"), a + b, "Addition".into())
        }
        BaseOp::Sub => {
            let (min, max) = match cfg.difficulty {
                Difficulty::Easy => (0, 9),
                Difficulty::Moderate => (10, 99),
                Difficulty::Advanced => (100, 999),
            };
            let a = rand_int(min, max);
            let b = rand_int(0, a); // ensure non-negative
            (format!("{a} − {b} = ?"), a - b, "Subtraction".into())
        }
        BaseOp::Mul => {
            // keep multiplication friendly but scaled
            let (min_f, max_f) = match cfg.difficulty {
                Difficulty::Easy => (0, 5),   // times tables 0–5
                Difficulty::Moderate => (2, 12),
                Difficulty::Advanced => (5, 20),
            };
            let a = rand_int(min_f, max_f);
            let b = rand_int(min_f, max_f);
            (format!("{a} × {b} = ?"), a * b, "Multiplication".into())
        }
        BaseOp::Div => {
            // Whole-number division, scaled by difficulty
            let (min_q, max_q) = match cfg.difficulty {
                Difficulty::Easy => (1, 9),
                Difficulty::Moderate => (2, 12),
                Difficulty::Advanced => (5, 20),
            };
            let divisor = rand_int(1, max_q);
            let quotient = rand_int(min_q, max_q);
            let dividend = divisor * quotient;
            (
                format!("{dividend} ÷ {divisor} = ?"),
                quotient,
                "Division".into(),
            )
        }
    }
}

/// Local fallback word problem, in case AI call fails
fn generate_fallback_word_problem(cfg: &QuizConfig) -> (String, i32, String) {
    let a = rand_int(3, 15);
    let b = rand_int(2, 10);
    let prompt = format!(
        "Kiki has {a} stickers. She gets {b} more from a friend. \
         How many stickers does Kiki have now?"
    );
    let answer = a + b;
    (
        prompt,
        answer,
        format!("Word Problem 🌟 ({})", difficulty_label(&cfg.difficulty)),
    )
}

/// Generate questions, but use placeholder rows for AI word problems.
/// Guarantee at least 1 AI word problem if cfg.include_words == true.
fn generate_questions_with_ai_placeholders(cfg: &QuizConfig) -> Vec<Question> {
    let mut enabled_ops = Vec::new();
    if cfg.include_add {
        enabled_ops.push(BaseOp::Add);
    }
    if cfg.include_sub {
        enabled_ops.push(BaseOp::Sub);
    }
    if cfg.include_mul {
        enabled_ops.push(BaseOp::Mul);
    }
    if cfg.include_div {
        enabled_ops.push(BaseOp::Div);
    }

    if enabled_ops.is_empty() && !cfg.include_words {
        // force at least addition if nothing chosen
        enabled_ops.push(BaseOp::Add);
    }

    let mut questions = Vec::with_capacity(cfg.num_questions);
    let mut ai_count = 0;

    for _ in 0..cfg.num_questions {
        let ai_word_enabled = cfg.include_words;
        let make_word = ai_word_enabled && rand_int(0, 3) == 0; // ~25%

        let (prompt, answer, kind) = if make_word {
            ai_count += 1;
            (
                "Loading AI word problem...".to_string(),
                0,
                format!("Word Problem 🌟 ({})", difficulty_label(&cfg.difficulty)),
            )
        } else {
            let idx = rand_int(0, (enabled_ops.len() as i32) - 1) as usize;
            let op = enabled_ops[idx];
            generate_basic_question(cfg, op)
        };

        questions.push(Question {
            prompt,
            kind,
            answer,
            user_answer: String::new(),
            is_correct: None,
        });
    }

    // Guarantee at least 1 AI word problem if enabled
    if cfg.include_words && ai_count == 0 {
        if let Some(first) = questions.get_mut(0) {
            first.prompt = "Loading AI word problem...".to_string();
            first.answer = 0;
            first.kind = format!("Word Problem 🌟 ({})", difficulty_label(&cfg.difficulty));
        }
    }

    questions
}

/// Call your Cloudflare Worker to get a word problem
/// Also update max_number to match difficulty tiers:
/// Easy: 9, Moderate: 99, Advanced: 999
async fn fetch_ai_word_problem(cfg: &QuizConfig) -> Option<(String, i32, String)> {
    let max_number = match cfg.difficulty {
        Difficulty::Easy => 9,
        Difficulty::Moderate => 99,
        Difficulty::Advanced => 999,
    };

    let body = AiWordProblemRequest {
        difficulty: difficulty_code(&cfg.difficulty).to_string(),
        max_number,
    };

    let resp = Request::post(AI_WORKER_URL)
        .header("Content-Type", "application/json")
        .json(&body)
        .ok()?
        .send()
        .await
        .ok()?;

    if !resp.ok() {
        log("fetch_ai_word_problem: non-OK HTTP from Worker");
        return None;
    }

    let data: AiWordProblemResponse = resp.json().await.ok()?;
    log("fetch_ai_word_problem: got JSON from Worker");
    Some((
        data.prompt,
        data.answer,
        format!("Word Problem 🌟 ({})", difficulty_label(&cfg.difficulty)),
    ))
}

#[function_component(App)]
fn app() -> Html {
    let config = use_state(default_config);
    let questions = use_state(Vec::<Question>::new);
    let show_results = use_state(|| false);
    let score = use_state(|| (0usize, 0usize)); // (correct, total)
    let teacher_mode = use_state(|| false);

    // === Config handlers ===

    let on_num_questions = {
        let config = config.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            let val = input.value().parse::<usize>().unwrap_or(10);
            let clamped = val.clamp(5, 20);
            let mut c = (*config).clone();
            c.num_questions = clamped;
            config.set(c);
        })
    };

    let on_difficulty_change = {
        let config = config.clone();
        Callback::from(move |e: Event| {
            let select: HtmlInputElement = e.target_unchecked_into();
            let value = select.value();
            let mut c = (*config).clone();
            c.difficulty = match value.as_str() {
                "moderate" => Difficulty::Moderate,
                "advanced" => Difficulty::Advanced,
                _ => Difficulty::Easy,
            };
            config.set(c);
        })
    };

    let toggle_checkbox = |field: &'static str,
                           config: UseStateHandle<QuizConfig>|
     -> Callback<InputEvent> {
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            let checked = input.checked();
            let mut c = (*config).clone();
            match field {
                "add" => c.include_add = checked,
                "sub" => c.include_sub = checked,
                "mul" => c.include_mul = checked,
                "div" => c.include_div = checked,
                "words" => c.include_words = checked,
                _ => {}
            }
            config.set(c);
        })
    };

    let on_add = toggle_checkbox("add", config.clone());
    let on_sub = toggle_checkbox("sub", config.clone());
    let on_mul = toggle_checkbox("mul", config.clone());
    let on_div = toggle_checkbox("div", config.clone());
    let on_words = toggle_checkbox("words", config.clone());

    // Teacher mode toggle
    let on_teacher_mode = {
        let teacher_mode = teacher_mode.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            teacher_mode.set(input.checked());
        })
    };

    // === Generate quiz (single async flow) ===

    let on_generate = {
        let config_handle = config.clone();
        let questions_state = questions.clone();
        let show_results = show_results.clone();
        let score = score.clone();

        Callback::from(move |_| {
            let cfg = (*config_handle).clone();
            let questions_state = questions_state.clone();
            let show_results = show_results.clone();
            let score = score.clone();

            spawn_local(async move {
                log("on_generate: building quiz with placeholders");
                let mut qs = generate_questions_with_ai_placeholders(&cfg);
                let total = qs.len();

                // Collect AI indexes from this local vec
                let ai_indexes: Vec<usize> = qs
                    .iter()
                    .enumerate()
                    .filter(|(_, q)| q.kind.contains("Word Problem"))
                    .map(|(i, _)| i)
                    .collect();

                log(&format!(
                    "on_generate: created {} questions; {} AI placeholders",
                    total,
                    ai_indexes.len()
                ));

                // Show the base quiz with "Loading AI word problem..."
                questions_state.set(qs.clone());
                show_results.set(false);
                score.set((0, total));

                // Now fill AI questions sequentially
                for idx in ai_indexes {
                    log(&format!("AI fill: idx {} -> calling Worker", idx));
                    let replacement = fetch_ai_word_problem(&cfg).await;
                    let (prompt, answer, kind) = if let Some(ok) = replacement {
                        log(&format!("AI fill: idx {} -> got AI result", idx));
                        ok
                    } else {
                        log(&format!("AI fill: idx {} -> using fallback", idx));
                        generate_fallback_word_problem(&cfg)
                    };

                    if let Some(q) = qs.get_mut(idx) {
                        q.prompt = prompt;
                        q.answer = answer;
                        q.kind = kind;
                        // Push updated snapshot to state
                        questions_state.set(qs.clone());
                    } else {
                        log(&format!("AI fill: idx {} out of range on local vec", idx));
                    }
                }
            });
        })
    };

    // === Regenerate a single AI question ===

    let on_regen_ai = {
        let config_handle = config.clone();
        let questions_state = questions.clone();

        Callback::from(move |idx: usize| {
            let cfg = (*config_handle).clone();
            let questions_state = questions_state.clone();

            spawn_local(async move {
                log(&format!("Regen: idx {} -> calling Worker", idx));
                let replacement = fetch_ai_word_problem(&cfg).await;
                let (prompt, answer, kind) = if let Some(ok) = replacement {
                    log(&format!("Regen: idx {} -> got AI result", idx));
                    ok
                } else {
                    log(&format!("Regen: idx {} -> using fallback", idx));
                    generate_fallback_word_problem(&cfg)
                };

                let mut qs = (*questions_state).clone();
                if let Some(q) = qs.get_mut(idx) {
                    q.prompt = prompt;
                    q.answer = answer;
                    q.kind = kind;
                    questions_state.set(qs);
                } else {
                    log(&format!("Regen: idx {} out of range on update", idx));
                }
            });
        })
    };

    // === Reset answers ===

    let on_reset_answers = {
        let questions_state = questions.clone();
        let show_results = show_results.clone();
        let score = score.clone();
        Callback::from(move |_| {
            let mut qs = (*questions_state).clone();
            for q in &mut qs {
                q.user_answer.clear();
                q.is_correct = None;
            }
            let total = qs.len();
            questions_state.set(qs);
            show_results.set(false);
            score.set((0, total));
        })
    };

    // === Grade quiz ===

    let on_check_answers = {
        let questions_state = questions.clone();
        let show_results = show_results.clone();
        let score = score.clone();
        Callback::from(move |_| {
            let mut qs = (*questions_state).clone();
            let mut correct = 0usize;
            let total = qs.len();
            for q in &mut qs {
                let trimmed = q.user_answer.trim();
                if let Ok(val) = trimmed.parse::<i32>() {
                    let ok = val == q.answer;
                    if ok {
                        correct += 1;
                    }
                    q.is_correct = Some(ok);
                } else {
                    q.is_correct = Some(false);
                }
            }
            score.set((correct, total));
            questions_state.set(qs);
            show_results.set(true);
        })
    };

    // === Print quiz ===

    let on_print = {
        let teacher_mode = teacher_mode.clone();
        Callback::from(move |_| {
            // Optional: nudge them to turn on teacher mode before printing
            log(&format!("Print clicked (teacher_mode = {})", *teacher_mode));
            if let Some(win) = web_sys::window() {
                let _ = win.print();
            }
        })
    };

    let (correct_count, total_count) = *score;

    html! {
        <div class="app-shell">
            <div class="card">
                <h1>{"Math Quest 🎒"}</h1>
                <div class="subtitle">
                    {"Build a custom 2nd–3rd grade math quiz with 5–20 questions, "}
                    {"including AI-generated word problems that match the difficulty."}
                </div>

                <div class="config-grid">
                    <div>
                        <div class="field-label">
                            <span>{"Number of questions"}</span>
                            <span class="field-hint">{"5 to 20"}</span>
                        </div>
                        <input
                            class="field-input"
                            type="number"
                            min="5"
                            max="20"
                            value={config.num_questions.to_string()}
                            oninput={on_num_questions}
                        />
                    </div>

                    <div>
                        <div class="field-label">
                            <span>{"Difficulty"}</span>
                        </div>
                        <select
                            class="field-input"
                            onchange={on_difficulty_change}
                            value={
                                match config.difficulty {
                                    Difficulty::Easy => "easy",
                                    Difficulty::Moderate => "moderate",
                                    Difficulty::Advanced => "advanced",
                                }.to_string()
                            }
                        >
                            <option value="easy">{"Easy – single-digit + small ×/÷"}</option>
                            <option value="moderate">{"Moderate – two-digit + bigger ×/÷"}</option>
                            <option value="advanced">{"Advanced – three-digit + challenge ×/÷"}</option>
                        </select>
                    </div>

                    <div>
                        <div class="field-label">
                            <span>{"Question types"}</span>
                        </div>
                        <div class="checkbox-row">
                            <input type="checkbox" checked={config.include_add} oninput={on_add} />
                            <span>{"+ (Add)"}</span>
                        </div>
                        <div class="checkbox-row">
                            <input type="checkbox" checked={config.include_sub} oninput={on_sub} />
                            <span>{"− (Subtract)"}</span>
                        </div>
                        <div class="checkbox-row">
                            <input type="checkbox" checked={config.include_mul} oninput={on_mul} />
                            <span>{"× (Multiply)"}</span>
                        </div>
                        <div class="checkbox-row">
                            <input type="checkbox" checked={config.include_div} oninput={on_div} />
                            <span>{"÷ (Divide)"}</span>
                        </div>
                    </div>

                    <div>
                        <div class="field-label">
                            <span>{"Extras"}</span>
                        </div>
                        <div class="checkbox-row">
                            <input type="checkbox" checked={config.include_words} oninput={on_words} />
                            <span>{"Include AI word problems"}</span>
                        </div>
                        <div class="checkbox-row">
                            <input type="checkbox" checked={*teacher_mode} oninput={on_teacher_mode} />
                            <span>{"Teacher mode (show answers & print)"}</span>
                        </div>
                        <div class="tiny-note">
                            {"Word problems come from your Cloudflare/OpenAI Worker; "}
                            {"if it fails, a local backup problem is used."}
                        </div>
                    </div>
                </div>

                <div class="btn-row">
                    <button class="btn-primary" onclick={on_generate}>
                        {"Generate Quiz"}
                    </button>
                    <button class="btn-secondary" onclick={on_check_answers}>
                        {"Check Answers"}
                    </button>
                    <button class="btn-secondary" onclick={on_reset_answers}>
                        {"Clear Answers"}
                    </button>
                    <button class="btn-secondary" onclick={on_print}>
                        {"Print Quiz"}
                    </button>
                </div>
                <div class="tiny-note">
                    {"All answers are whole numbers—perfect for 2nd and 3rd graders."}
                </div>
            </div>

            <div class="card">
                <h2>{"Your Quiz"}</h2>

                if questions.is_empty() {
                    <p>{"Click “Generate Quiz” to create a new set of questions."}</p>
                } else {
                    <div class="questions-wrap">
                        { for questions.iter().enumerate().map(|(idx, q)| {
                            let idx_copy = idx;
                            let questions_state = questions.clone();
                            html! {
                                <QuestionRow
                                    index={idx_copy}
                                    question={q.clone()}
                                    questions_state={questions_state}
                                    show_results={*show_results}
                                    teacher_mode={*teacher_mode}
                                    on_regen_ai={on_regen_ai.clone()}
                                />
                            }
                        }) }
                    </div>

                    if *show_results {
                        <div class="score-banner">
                            <div>
                                <span class="score-main">
                                    {format!("Score: {}/{}", correct_count, total_count)}
                                </span>
                                {"  "}
                                {
                                    if total_count > 0 {
                                        let pct = (correct_count as f64 / total_count as f64 * 100.0).round() as i32;
                                        format!("({}% correct)", pct)
                                    } else {
                                        "".into()
                                    }
                                }
                            </div>
                            <div class="tiny-note">
                                {
                                    if correct_count == total_count && total_count > 0 {
                                        "Perfect score! 🏆"
                                    } else if correct_count * 2 >= total_count {
                                        "Nice work! Look over the ones marked in red and try again. 💪"
                                    } else {
                                        "Great practice round. Try a new quiz or pick an easier level and build up! 🌱"
                                    }
                                }
                            </div>
                        </div>
                    }
                }
            </div>

            <div class="tiny-note">
                {"Security note: your OpenAI key stays in Cloudflare; this app calls only your Worker URL."}
            </div>
        </div>
    }
}

#[derive(Properties, PartialEq)]
struct QuestionRowProps {
    index: usize,
    question: Question,
    questions_state: UseStateHandle<Vec<Question>>,
    show_results: bool,
    teacher_mode: bool,
    on_regen_ai: Callback<usize>,
}

#[function_component(QuestionRow)]
fn question_row(props: &QuestionRowProps) -> Html {
    let index: usize = props.index;
    let question: Question = props.question.clone();
    let questions_state = props.questions_state.clone();
    let show_results: bool = props.show_results;
    let teacher_mode: bool = props.teacher_mode;
    let on_regen_ai = props.on_regen_ai.clone();

    let is_word = question.kind.contains("Word Problem");

    let on_answer_change = {
        let questions_state = questions_state.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            let value = input.value();
            let mut qs = (*questions_state).clone();
            if let Some(q) = qs.get_mut(index) {
                q.user_answer = value;
                q.is_correct = None;
            }
            questions_state.set(qs);
        })
    };

    let on_regen_click = {
        let on_regen_ai = on_regen_ai.clone();
        let idx = index;
        Callback::from(move |_| {
            on_regen_ai.emit(idx);
        })
    };

    let feedback = if show_results {
        if let Some(is_correct) = question.is_correct {
            if is_correct {
                html! { <div class="feedback correct">{"✅ Nice job!"}</div> }
            } else {
                html! {
                    <div class="feedback incorrect">
                        {format!("❌ Not quite. Correct answer: {}", question.answer)}
                    </div>
                }
            }
        } else {
            Html::default()
        }
    } else {
        Html::default()
    };

    let teacher_answer = if teacher_mode {
        html! {
            <div class="teacher-answer">
                {format!("Answer (teacher): {}", question.answer)}
            </div>
        }
    } else {
        Html::default()
    };

    html! {
        <div class="question-card">
            <div class="question-header">
                <div class="question-index">
                    {format!("Question {}", index + 1)}
                </div>
                <div class={classes!(
                    "question-tag",
                    if is_word { "tag-word" } else { "tag-basic" }
                )}>
                    {question.kind.clone()}
                </div>
            </div>
            <div class="question-text">
                {question.prompt.clone()}
            </div>
            <div class="answer-row">
                <input
                    class="answer-input"
                    type="number"
                    inputmode="numeric"
                    placeholder="Your answer"
                    value={question.user_answer.clone()}
                    oninput={on_answer_change}
                />
                { if is_word {
                    html! {
                        <button class="btn-regen" onclick={on_regen_click}>
                            {"Regenerate 🔁"}
                        </button>
                    }
                } else {
                    Html::default()
                }}
            </div>
            {feedback}
            {teacher_answer}
        </div>
    }
}

// Trunk/Yew entrypoint
fn main() {
    yew::Renderer::<App>::new().render();
}
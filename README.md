➗ Math Quiz Yew

A Rust + WebAssembly adaptive math practice tool for kids

A fast, distraction-free math practice experience built using Rust, WebAssembly, and Yew — designed for home learning, classrooms, and Chromebook environments.

🎯 Purpose

Math Quiz Yew was built to help kids (and students of any age) build fluency in:

Addition

Subtraction

Multiplication

Division

Mixed practice drills

The goal: short, repeatable practice sessions that build confidence — not frustration.

This project is parent-tested, teacher-minded, and optimized for quick sessions, scores, and retry loops.

🧠 Key Features
Feature	Status
🧮 Random questions by difficulty level	✔️
🕹️ Simple and fast UI (one input, one button)	✔️
📊 Score tracking with accuracy percent	✔️
🔁 Retry same set or generate a new one	✔️
📱 Mobile + Chromebook friendly	✔️
⚡ Instant WASM performance	✔️
🎨 Kid-friendly layout + readability focus	✔️
🚫 No ads, no database, no tracking	✔️
🧰 Tech Stack
Layer	Technology
Frontend framework	🦀 Rust + Yew
Execution model	WebAssembly
Build tool	trunk + wasm-bindgen
Hosting options	Hostek, Cloudflare Pages, GitHub Pages, Netlify
📦 Installation & Development
1️⃣ Install Rust & Target
rustup update
rustup target add wasm32-unknown-unknown

2️⃣ Install Trunk
cargo install trunk

3️⃣ Clone the Repository
git clone https://github.com/<your-name>/math-quiz-yew
cd math-quiz-yew

4️⃣ Run Dev Server
trunk serve --open

5️⃣ Build Production Version
trunk build --release


The optimized build will appear in:

/dist


Upload this folder directly to any static web host.

🧪 Testing Scenarios

Use the following to confirm everything behaves correctly:

Scenario	Expected Behavior
No input submitted	App should warn user instead of marking wrong
Wrong answer submitted	Record attempt count and stay on question
Correct answer submitted	Auto-advance to next question
End of quiz	Show score summary + retry options
Retry same set	Uses same questions with attempts reset
🧩 App Flow

User selects:

Operation type (add/multiply/etc.)

Difficulty range (e.g., 1–10, 1–20, etc.)

Number of questions

App generates randomized questions.

User answers one at a time — feedback is instant.

Results screen shows:

Total correct

Attempts

Final percentage

User chooses:

🔁 Retry same quiz

🔄 Start a new one

🌱 Future Enhancements
Planned Feature	Priority
🔊 Voice read-aloud mode for early learners	⭐⭐⭐
🎉 Badge achievements (10 in a row, no mistakes, etc.)	⭐⭐
🧮 Timed practice mode	⭐⭐
🎨 Theme options (dark mode, dyslexia font, NASA theme 🚀)	⭐
👨‍🏫 Teacher mode with printable report	⭐
👪 Who It Was Built For

Originally designed for a younger learner, tested by an older sibling, and improved through feedback from:

Parents

Teachers

District tech staff

Rust & WASM dev community

This isn't just software — it's a learning journey turned open-source project.

📄 License

MIT License

Permission free to use at home, in school, or fork for EdTech research or classroom pilots.

🤝 Contributing

Pull requests are welcome — whether you're:

A Rust developer

A teacher with feature ideas

A UI/UX person who loves kid-friendly apps

A student learning WASM

Open an issue or PR and join the project.

⭐ Support

If you'd like to support the project:

⭐ Star the repo

Share with a teacher or homeschool parent

Pilot it in a classroom

Send feedback

🧮 Keep learning — one correct answer at a time!

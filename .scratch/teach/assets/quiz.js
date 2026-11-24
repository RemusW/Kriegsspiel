/*
 * Reusable quiz component.
 *
 * Declarative: write the questions as HTML and this script makes them interactive.
 * Feedback is immediate and per-question, which is the point: the loop between answering
 * and finding out has to be as tight as possible.
 *
 *   <ol class="quiz" data-label="Recall">      <!-- add class "two" for 2-option drills -->
 *     <li class="q" data-answer="2">           <!-- 1-based index of the correct option -->
 *       <p class="prompt">Why does X happen?</p>
 *       <ul class="options">
 *         <li>Because of A</li>
 *         <li>Because of B</li>
 *       </ul>
 *       <p class="why">Shown once answered. Explain <strong>why</strong>, not just what.</p>
 *     </li>
 *   </ol>
 *
 * Each question can be answered once. A running score appears under the list.
 * No dependencies, no build step: lessons are files you open straight from disk.
 */
(function () {
  function setup(quiz) {
    var questions = Array.prototype.slice.call(quiz.querySelectorAll(".q"));
    if (!questions.length) return;

    var label = quiz.dataset.label || "Answered";
    var score = document.createElement("p");
    score.className = "score";
    quiz.appendChild(score);

    var answered = 0;
    var right = 0;

    function render() {
      score.textContent =
        label + ": " + answered + " of " + questions.length +
        (answered ? "  ·  " + right + " correct" : "");
      if (answered === questions.length) score.classList.add("done");
    }

    questions.forEach(function (q) {
      var options = Array.prototype.slice.call(q.querySelectorAll(".options > li"));
      var correct = parseInt(q.dataset.answer, 10) - 1;

      options.forEach(function (option, index) {
        // Options are <li>, so they need the role and tab stop a button would have had.
        option.setAttribute("role", "button");
        option.tabIndex = 0;

        function choose() {
          if (q.classList.contains("answered")) return;
          q.classList.add("answered");
          answered += 1;
          if (index === correct) right += 1;

          options.forEach(function (other, i) {
            other.setAttribute("aria-disabled", "true");
            other.tabIndex = -1;
            if (i === correct) other.dataset.state = "right";
            else if (i === index) other.dataset.state = "wrong";
            else other.dataset.state = "muted";
          });

          render();
        }

        option.addEventListener("click", choose);
        option.addEventListener("keydown", function (event) {
          if (event.key === "Enter" || event.key === " ") {
            event.preventDefault();
            choose();
          }
        });
      });
    });

    render();
  }

  function init() {
    document.querySelectorAll(".quiz").forEach(setup);
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", init);
  } else {
    init();
  }
})();

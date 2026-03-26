import Prism from 'prismjs';
import initWasm, { run_laadle_code } from './wasm/laadlelang.js';

// Define the LaadleLang grammar for PrismJS based on TextMate spec
Prism.languages.laadle = {
    'comment': /\/\/.*$/,
    'string': {
        pattern: /"(?:\\.|[^\\"\r\n])*"/,
        greedy: true
    },
    'keyword': /\b(?:agar|warna|jabtak|koshish|pakad|wapas|nikal|aage|laadle|kaam|hai|toh)\b/,
    'function': /\b(?:bol|gopgop)\b/,
    'boolean': /\b(?:sahi|galat)\b/,
    'null': /\bmeow\b/,
    'number': /\b\d+(?:\.\d+)?\b/,
    'operator': /[+\-*\/=<>!|&]=?/,
    'punctuation': /[{}[\];(),.:]/
};

const inputArea = document.getElementById('input-area');
const highlightArea = document.getElementById('highlight-area');
const highlightCode = document.getElementById('highlight-code');
const runBtn = document.getElementById('run-btn');
const btnText = document.getElementById('btn-text');
const output = document.getElementById('output');
const loading = document.getElementById('loading');

let isWasmLoaded = false;

function syncScroll() {
    highlightArea.scrollTop = inputArea.scrollTop;
    highlightArea.scrollLeft = inputArea.scrollLeft;
}

function updateHighlight() {
    let code = inputArea.value;
    // Add space for trailing newline so the height calculates correctly
    if (code[code.length - 1] === "\n") {
        code += " ";
    }
    const html = Prism.highlight(code, Prism.languages.laadle, 'laadle');
    highlightCode.innerHTML = html;
}

inputArea.addEventListener('input', updateHighlight);
inputArea.addEventListener('scroll', syncScroll);

// Tab support in textarea
inputArea.addEventListener('keydown', (e) => {
    if (e.key === 'Tab') {
        e.preventDefault();
        const start = inputArea.selectionStart;
        const end = inputArea.selectionEnd;
        inputArea.value = inputArea.value.substring(0, start) + '    ' + inputArea.value.substring(end);
        inputArea.selectionStart = inputArea.selectionEnd = start + 4;
        updateHighlight();
    }
});

runBtn.addEventListener('click', () => {
    if (!isWasmLoaded) return;
    output.textContent = 'Compiling...';
    setTimeout(() => {
        try {
            const res = run_laadle_code(inputArea.value);
            output.textContent = res || '✅ Execution finished.';
        } catch (err) {
            output.textContent = '🚨 Error: ' + err;
        }
    }, 50);
});

async function start() {
    try {
        await initWasm();
        isWasmLoaded = true;
        loading.style.display = 'none';
        runBtn.disabled = false;
        btnText.textContent = 'Run';
        updateHighlight();
    } catch (err) {
        loading.textContent = '❌ Failed to load WASM VM: ' + err;
        console.error(err);
    }
}

start();

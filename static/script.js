const canvas = document.querySelector("canvas");
const ctx = canvas.getContext("2d");
let width = (canvas.width = window.innerWidth);
let height = (canvas.height = window.innerHeight);

const playx = 2000;
const playy = 500;

let you = "";

let bullets = [];
let players = [];
let demon = {};

let ws = new WebSocket("ws://" + window.location.host + "/ws/");
let opened = false;
let myid = 0;

function send(m) {
    if (opened) ws.send(JSON.stringify(m));
}

document.getElementById("username").focus();
document.getElementById("username").addEventListener("keydown", e => {
    if (e.keyCode == 13 && opened) {
        send({ Name: document.getElementById("username").value });
        document.getElementById("login").style.display = "none";
    }
});
ws.addEventListener("open", () => {
    opened = true;
    document.getElementById("status").innerText = "Press enter to play";
});
ws.addEventListener("close", () => (opened = false));
ws.addEventListener("message", e => {
    const m = JSON.parse(e.data);

    if (m.you) you = m.you;
    if (m.players) players = m.players;
    if (m.bullets) bullets = m.bullets;
    if (m.demon) demon = m.demon;
    if (m.death == you) {
        document.getElementById("login").style.display = null;
        document.getElementById("username").focus();
    }
});

let tick = 0;
function game() {
    const subx = players[you] ? players[you].pos[0] - width / 2 : 0;
    const suby = players[you] ? players[you].pos[1] - height / 2 : 0;

    ctx.resetTransform();
    ctx.clearRect(0, 0, width, height);
    const x = players[you] ? -players[you].pos[0] % 50 : 0;
    const y = players[you] ? -players[you].pos[1] % 50 : 0;
    ctx.lineWidth = 1;
    for (var i = x; i < window.innerWidth; i += 50) {
        ctx.beginPath();
        ctx.moveTo(i, 0);
        ctx.lineTo(i, window.innerHeight);
        ctx.stroke();
    }
    for (var j = y; j < window.innerHeight; j += 50) {
        ctx.beginPath();
        ctx.moveTo(0, j);
        ctx.lineTo(window.innerWidth, j);
        ctx.stroke();
    }
    ctx.translate(-subx, -suby);

    ctx.lineWidth = 5;
    ctx.strokeStyle = "#aaa";
    ctx.beginPath();
    ctx.moveTo(0, 0);
    ctx.lineTo(playx, 0);
    ctx.lineTo(playx, playy);
    ctx.lineTo(0, playy);
    ctx.lineTo(0, 0);
    ctx.stroke();

    for (let p of Object.values(players)) {
        ctx.strokeStyle = "#aaa";
        ctx.beginPath();
        ctx.moveTo(p.pos[0], p.pos[1]);
        ctx.lineTo(p.anchor[0], p.anchor[1]);
        ctx.stroke();

        if (p.shielding) {
            let ox = Math.cos(p.angle),
                oy = Math.sin(p.angle);
            ctx.setLineDash([10, 20]);
            ctx.beginPath();
            ctx.moveTo(p.pos[0], p.pos[1]);
            ctx.lineTo(p.pos[0] + ox * 400, p.pos[1] + oy * 400);
            ctx.stroke();
            ctx.setLineDash([]);
        }

        ctx.fillStyle = "#aaa";
        ctx.textAlign = "center";
        ctx.font = "30px monospace";
        ctx.fillText(p.name, p.pos[0], p.pos[1] + 50);

        ctx.fillStyle = p.shielding ? "#aaa" : `hsl(${tick * 2}, 75%, 50%)`;
        ctx.strokeStyle = p.shielding ? `hsl(${tick * 2}, 75%, 50%)` : "#aaa";
        ctx.beginPath();
        ctx.arc(p.pos[0], p.pos[1], 15, 0, 2 * Math.PI);
        ctx.fill();
        ctx.stroke();
    }
    for (let bg in bullets) {
        for (var b of bullets[bg]) {
            ctx.fillStyle = bg == you ? "blue" : "red";
            ctx.beginPath();
            ctx.arc(b.pos[0], b.pos[1], 5, 0, 2 * Math.PI);
            ctx.fill();
        }
    }

    if (demon.pos) {
        ctx.fillStyle = "#aaa";
        ctx.fillRect(demon.pos[0] - 25, demon.pos[1] - 50, 50, 100);
        ctx.fillStyle = `hsl(${Math.floor(demon.health * 0.47)}, 100%, 50%)`;
        ctx.fillRect(demon.pos[0] - 25, demon.pos[1] - 70, (demon.health / 255) * 50, 10);
    }

    tick++;
    window.requestAnimationFrame(game);
}
game();

window.addEventListener("mousedown", e => {
    if (e.button == 0) send({ Shoot: true });
    else send({ Shield: true });
});
window.addEventListener("mousemove", e => {
    send({ Angle: Math.atan2(e.clientY - height / 2, e.clientX - width / 2) });
});
window.addEventListener("mouseup", e => {
    if (e.button == 0) send({ Shoot: false });
    if (e.button == 2) send({ Shield: false });
});
window.addEventListener("keydown", e => {
    if (e.keyCode == 32) send({ Shoot: true });
});
window.addEventListener("keyup", e => {
    if (e.keyCode == 32) send({ Shoot: false });
});
window.addEventListener("contextmenu", e => {
    e.preventDefault();
});
window.addEventListener("resize", () => {
    width = canvas.width = window.innerWidth;
    height = canvas.height = window.innerHeight;
});

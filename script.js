const canvas = document.querySelector("canvas");
const ctx = canvas.getContext("2d");
let width = (canvas.width = window.innerWidth);
let height = (canvas.height = window.innerHeight);

const playx = 2000;
const playy = 1000;

const you = {
    x: playx / 2,
    y: playy / 2,
    vx: 0,
    vy: 0,
    shielding: false
};
const mouse = {
    x: 0,
    y: 0
};
const anchor = {
    x: playx / 2,
    y: playy / 2 - 215
};
let bullets = [];

let tick = 0;
function game() {
    let x = anchor.x - you.x - you.vx,
        y = anchor.y - you.y - you.vy,
        l = Math.sqrt(x * x + y * y);
    l = ((l - 200) / l) * 0.01;
    x *= l;
    y *= l;
    you.vx += x;
    you.vy += y;

    you.vy += 0.15;

    you.x += you.vx *= 0.98;
    you.y += you.vy *= 0.98;

    if (you.x > playx) {
        you.x = playx;
        you.vx = -you.vx;
    }
    if (you.x < 0) {
        you.x = 0;
        you.vx = -you.vx;
    }
    if (you.y > playy) {
        you.y = playy;
        you.vy = -you.vy;
    }
    if (you.y < 0) {
        you.y = 0;
        you.vy = -you.vy;
    }

    if (you.shielding) {
        let ox = mouse.x - width / 2,
            oy = mouse.y - height / 2,
            len = Math.sqrt(ox * ox + oy * oy);
        ox /= len;
        oy /= len;
        anchor.x = you.x + ox * 200;
        anchor.y = you.y + oy * 200;
    }

    bullets = bullets.filter(b => b.time + 5000 > Date.now());
    for (let b of bullets) {
        b.vy += 0.05;
        b.x += b.vx *= 0.999;
        b.y += b.vy *= 0.999;
        if (b.x > playx) {
            b.x = playx;
            b.vx = -b.vx;
        }
        if (b.x < 0) {
            b.x = 0;
            b.vx = -b.vx;
        }
        if (b.y > playy) {
            b.y = playy;
            b.vy = -b.vy;
        }
        if (b.y < 0) {
            b.y = 0;
            b.vy = -b.vy;
        }
    }
    const subx = you.x - width / 2;
    const suby = you.y - height / 2;

    ctx.resetTransform();
    ctx.clearRect(0, 0, width, height);
    ctx.translate(-subx, -suby);

    ctx.strokeStyle = "#aaa";
    ctx.beginPath();
    ctx.moveTo(0, 0);
    ctx.lineTo(playx, 0);
    ctx.lineTo(playx, playy);
    ctx.lineTo(0, playy);
    ctx.lineTo(0, 0);
    ctx.stroke();

    ctx.strokeStyle = "#aaa";
    ctx.beginPath();
    ctx.moveTo(you.x, you.y);
    ctx.lineTo(anchor.x, anchor.y);
    ctx.stroke();

    if (you.shielding) {
        let ox = mouse.x - width / 2,
            oy = mouse.y - height / 2,
            len = Math.sqrt(ox * ox + oy * oy);
        ox /= len;
        oy /= len;
        ctx.setLineDash([10, 20]);
        ctx.beginPath();
        ctx.moveTo(you.x, you.y);
        ctx.lineTo(you.x + ox * 400, you.y + oy * 400);
        ctx.stroke();
        ctx.setLineDash([]);
    }

    for (let b of bullets) {
        ctx.fillStyle = "red";
        ctx.beginPath();
        ctx.arc(b.x, b.y, 5, 0, 2 * Math.PI);
        ctx.fill();
    }

    ctx.fillStyle = you.shielding ? "#aaa" : `hsl(${tick * 2}, 75%, 50%)`;
    ctx.strokeStyle = you.shielding ? `hsl(${tick * 2}, 75%, 50%)` : "#aaa";
    ctx.lineWidth = 5;
    ctx.beginPath();
    ctx.arc(you.x, you.y, 15, 0, 2 * Math.PI);
    ctx.fill();
    ctx.stroke();

    ctx.strokeStyle = "#aaa";
    ctx.beginPath();
    ctx.arc(playx / 2, playy / 2, 20, 0, 2 * Math.PI);
    ctx.stroke();
    tick++;
    window.requestAnimationFrame(game);
}
game();

let lastClick = 0;
let lastShield = 0;
function shoot() {
    if (lastClick + 500 < Date.now()) {
        let vx = mouse.x - width / 2,
            vy = mouse.y - height / 2,
            len = Math.sqrt(vx * vx + vy * vy);
        vx /= len;
        vy /= len;
        bullets.push({
            x: you.x,
            y: you.y,
            vx: you.vx + vx * 8,
            vy: you.vy + vy * 8,
            time: Date.now()
        });
        you.vx -= vx * 8;
        you.vy -= vy * 8;
        lastClick = Date.now();
    }
}
window.addEventListener("mousedown", e => {
    if (e.button == 2 && lastShield + 1000 < Date.now()) you.shielding = true;
    else shoot();
});
window.addEventListener("mousemove", e => {
    mouse.x = e.clientX;
    mouse.y = e.clientY;
});
window.addEventListener("mouseup", e => {
    if (e.button == 2 && you.shielding) {
        you.shielding = false;
        let ox = mouse.x - width / 2,
            oy = mouse.y - height / 2,
            len = Math.sqrt(ox * ox + oy * oy);
        ox /= len;
        oy /= len;
        anchor.x = you.x + ox * 400;
        anchor.y = you.y + oy * 400;
        lastShield = Date.now();
    }
});
window.addEventListener("keydown", e => {
    if (e.keyCode == 32) shoot();
});
window.addEventListener("contextmenu", e => {
    e.preventDefault();
});
window.addEventListener("resize", () => {
    width = canvas.width = window.innerWidth;
    height = canvas.height = window.innerHeight;
});

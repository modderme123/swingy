use actix::prelude::*;
use na::Vector2;
use nalgebra as na;
use rand::prelude::*;
use serde_derive::{Deserialize, Serialize};
use serde_json::json;
use std::cell::RefCell;
use std::collections::HashMap;
use std::f32::consts::PI;
use std::time::Duration;
use std::time::Instant;

const PLAYX: f32 = 4000.0;
const PLAYY: f32 = 750.0;

#[derive(Message)]
pub struct Message(pub String);

#[derive(Message)]
#[rtype(usize)]
pub struct Connect {
    pub addr: Recipient<Message>,
}

#[derive(Message, Serialize)]
pub struct Disconnect {
    pub id: usize,
}

#[derive(Deserialize)]
pub enum ClientMessage {
    Name(String),
    Angle(f32),
    Shoot(bool),
    Shield(bool),
}

#[derive(Message)]
pub struct ServerMessage {
    pub id: usize,
    pub m: ClientMessage,
}

pub struct Player {
    pub pos: Vector2<f32>,
    pub vel: Vector2<f32>,
    pub anchor: Vector2<f32>,
    pub angle: f32,
    pub health: u8,
    pub shielding: bool,
    pub shooting: bool,
    pub score: u32,
    pub name: String,
    pub last_shield: Instant,
    pub last_shot: Instant,
}

#[derive(Serialize, Copy, Clone)]
pub struct Demon {
    pub pos: Vector2<f32>,
    pub vel: Vector2<f32>,
    pub health: u8,
}

struct Bullet {
    pos: Vector2<f32>,
    vel: Vector2<f32>,
    time: Instant,
}

pub struct GameServer {
    sessions: HashMap<usize, Recipient<Message>>,
    players: HashMap<usize, Player>,
    bullets: HashMap<usize, Vec<Bullet>>,
    demon: Demon,
    tick: usize,
    rng: RefCell<ThreadRng>,
}

#[derive(Serialize)]
struct ClientPlayer {
    pos: Vector2<f32>,
    anchor: Vector2<f32>,
    name: String,
    angle: f32,
    health: u8,
    score: u32,
    shooting: bool,
    shielding: bool,
}

#[derive(Serialize)]
struct ClientBullet {
    pos: Vector2<f32>,
    vel: Vector2<f32>,
}

#[derive(Serialize)]
struct Playfield {
    players: HashMap<usize, ClientPlayer>,
    bullets: HashMap<usize, Vec<ClientBullet>>,
    demon: Demon,
}
impl GameServer {
    pub fn new() -> GameServer {
        let demon = Demon {
            pos: Vector2::new(50.0, PLAYY / 2.0),
            vel: Vector2::new(8.0, 0.0),
            health: 255,
        };
        GameServer {
            sessions: HashMap::new(),
            players: HashMap::new(),
            bullets: HashMap::new(),
            demon,
            tick: 0,
            rng: RefCell::new(rand::thread_rng()),
        }
    }
    fn send_message(&self, message: &str) {
        for addr in self.sessions.values() {
            let _ = addr.do_send(Message(message.to_owned()));
        }
    }
    fn tick(&self, ctx: &mut Context<Self>) {
        ctx.run_later(Duration::from_millis(16), |act, ctx| {
            {
                let d = &mut act.demon;
                if d.health == 0 {
                    d.pos.x = 0.0;
                    d.health = 255;
                }
                if act.tick % 3 == 0 {
                    d.health = d.health.saturating_add(1);
                }
                if act.tick % 100 == 0 {
                    for i in 0..30 {
                        let angle = i as f32 / 30.0 * PI - PI / 2.0 * d.vel.x.signum();
                        let vec = Vector2::new(angle.cos(), angle.sin());
                        let bullet = Bullet {
                            pos: d.pos,
                            vel: vec * 16.0,
                            time: Instant::now(),
                        };
                        if let Some(x) = act.bullets.get_mut(&0) {
                            x.push(bullet);
                        } else {
                            act.bullets.insert(0, vec![bullet]);
                        }
                    }
                }
                if d.pos.x > PLAYX {
                    d.pos.x = PLAYX;
                    d.vel.x *= -1.0;
                }
                if d.pos.x < 0.0 {
                    d.pos.x = 0.0;
                    d.vel.x *= -1.0;
                }
                d.pos += d.vel;
            }
            for (id, p) in act.players.iter_mut() {
                let mut x = p.anchor.x - p.pos.x - p.vel.x;
                let mut y = p.anchor.y - p.pos.y - p.vel.y;
                let mut l = (x * x + y * y).sqrt();
                l = ((l - 200.0) / l) * 0.004;
                x *= l;
                y *= l;
                p.vel.x += x;
                p.vel.y += y;

                p.vel.y += 0.2;

                p.vel *= 0.99;
                p.pos += p.vel;

                if p.pos.x > PLAYX {
                    p.pos.x = PLAYX;
                    p.vel.x *= -1.0;
                }
                if p.pos.x < 0.0 {
                    p.pos.x = 0.0;
                    p.vel.x *= -1.0;
                }
                if p.pos.y > PLAYY {
                    p.pos.y = PLAYY;
                    p.vel.y *= -1.0;
                }
                if p.pos.y < 0.0 {
                    p.pos.y = 0.0;
                    p.vel.y *= -1.0;
                }

                if act.tick % 3 == 0 {
                    p.health = p.health.saturating_add(1);
                }

                if p.shielding {
                    let ox = p.angle.cos();
                    let oy = p.angle.sin();
                    p.anchor.x = p.pos.x + ox * 200.0;
                    p.anchor.y = p.pos.y + oy * 200.0;
                }
                if p.pos.x < act.demon.pos.x + 25.0
                    && p.pos.x > act.demon.pos.x - 25.0
                    && p.pos.y < act.demon.pos.y + 50.0
                    && p.pos.y > act.demon.pos.y - 50.0
                {
                    if p.shielding {
                        p.health = p.health.saturating_sub(10);
                        act.demon.health = act.demon.health.saturating_sub(20);
                    } else {
                        p.health = p.health.saturating_sub(20);
                        act.demon.health = act.demon.health.saturating_sub(5);
                    }
                }
                if p.shooting && p.last_shot.elapsed().as_millis() > 500 {
                    let recoil = Vector2::new(p.angle.cos(), p.angle.sin()) * 8.0;
                    let bullet = Bullet {
                        pos: p.pos,
                        vel: recoil * 2.0,
                        time: Instant::now(),
                    };
                    if let Some(x) = act.bullets.get_mut(id) {
                        x.push(bullet);
                    } else {
                        act.bullets.insert(*id, vec![bullet]);
                    }
                    p.vel -= recoil;
                    p.last_shot = Instant::now();
                }
            }

            for (id, bg) in act.bullets.iter_mut() {
                for b in bg.iter_mut() {
                    b.vel.y += 0.05;
                    b.vel *= 0.999;
                    b.pos += b.vel;
                    if b.pos.x > PLAYX {
                        b.pos.x = PLAYX;
                        b.vel.x *= -1.0;
                    }
                    if b.pos.x < 0.0 {
                        b.pos.x = 0.0;
                        b.vel.x *= -1.0;
                    }
                    if b.pos.y > PLAYY {
                        b.pos.y = PLAYY;
                        b.vel.y *= -1.0;
                    }
                    if b.pos.y < 0.0 {
                        b.pos.y = 0.0;
                        b.vel.y *= -1.0;
                    }
                    if *id != 0 {
                        if b.pos.x < act.demon.pos.x + 25.0
                            && b.pos.x > act.demon.pos.x - 25.0
                            && b.pos.y < act.demon.pos.y + 50.0
                            && b.pos.y > act.demon.pos.y - 50.0
                        {
                            b.time -= Duration::from_secs(2);
                            if let Some(n) = act.demon.health.checked_sub(50) {
                                act.demon.health = n;
                            } else {
                                if let Some(p) = act.players.get_mut(id) {
                                    p.score += 1;
                                }
                            }
                        }
                    } else {
                        for p in act.players.values_mut() {
                            if (p.pos - b.pos).norm() < 20.0 {
                                p.health =
                                    p.health.saturating_sub(if p.shielding { 25 } else { 50 });
                                b.time -= Duration::from_secs(2);
                            }
                        }
                    }
                }

                bg.retain(|b| b.time.elapsed().as_millis() < 2000)
            }

            act.reap_players();

            act.tick += 1;

            let playfield = Playfield {
                players: act
                    .players
                    .iter()
                    .map(|(i, p)| {
                        (
                            *i,
                            ClientPlayer {
                                pos: p.pos,
                                anchor: p.anchor,
                                name: p.name.to_string(),
                                angle: p.angle,
                                health: p.health,
                                score: p.score,
                                shielding: p.shielding,
                                shooting: p.shooting,
                            },
                        )
                    })
                    .collect(),
                bullets: act
                    .bullets
                    .iter()
                    .map(|(i, p)| {
                        (
                            *i,
                            p.iter()
                                .map(|x| ClientBullet {
                                    pos: x.pos,
                                    vel: x.vel,
                                })
                                .collect(),
                        )
                    })
                    .collect(),
                demon: act.demon,
            };
            let serialized = ::serde_json::to_string(&playfield).unwrap();
            act.send_message(&serialized);

            act.tick(ctx);
        });
    }
    fn reap_players(&mut self) {
        let mut delete = Vec::new();
        self.players.retain(|i, p| {
            if p.health == 0 {
                delete.push(*i);
                false
            } else {
                true
            }
        });
        for p in &delete {
            self.send_message(
                &json!({
                    "death": p,
                })
                .to_string(),
            )
        }
    }
}

impl Actor for GameServer {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.tick(ctx);
    }
}

impl Handler<Connect> for GameServer {
    type Result = usize;

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
        let id = self.rng.borrow_mut().gen::<usize>();
        self.sessions.insert(id, msg.addr);

        id
    }
}

impl Handler<Disconnect> for GameServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        self.sessions.remove(&msg.id);
        self.players.remove(&msg.id);
        self.send_message(
            &json!({
                "death": msg.id
            })
            .to_string(),
        );
    }
}

impl Handler<ServerMessage> for GameServer {
    type Result = ();

    fn handle(&mut self, msg: ServerMessage, _: &mut Context<Self>) {
        if let Some(p) = self.players.get_mut(&msg.id) {
            match msg.m {
                ClientMessage::Shoot(s) => p.shooting = s,
                ClientMessage::Shield(s) if !s && p.shielding => {
                    p.anchor.x = p.pos.x + p.angle.cos() * 400.0;
                    p.anchor.y = p.pos.y + p.angle.sin() * 400.0;
                    p.last_shield = Instant::now();
                    p.shielding = false;
                }
                ClientMessage::Shield(s) => {
                    if p.last_shield.elapsed().as_millis() > 100 {
                        p.shielding = s
                    }
                }
                ClientMessage::Angle(a) => p.angle = a,
                ClientMessage::Name(_) => (),
            }
        } else {
            if let ClientMessage::Name(name) = msg.m {
                let p = Player {
                    vel: Vector2::new(0.0, 0.0),
                    pos: Vector2::new(PLAYX / 2.0, PLAYY / 2.0),
                    anchor: Vector2::new(PLAYX / 2.0, PLAYY / 2.0 - 250.0),
                    name,
                    score: 0,
                    angle: 0.0,
                    health: 255,
                    shielding: false,
                    shooting: false,
                    last_shield: Instant::now(),
                    last_shot: Instant::now(),
                };

                self.players.insert(msg.id, p);
            }
        }
    }
}

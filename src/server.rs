//! `GameServer` is an actor. It maintains list of connection client session.
//!  Peers send messages to other peers through `GameServer`.
use actix::prelude::*;
use na::Vector2;
use nalgebra as na;
use rand::prelude::*;
use serde_derive::{Deserialize, Serialize};
use serde_json::json;
use std::cell::RefCell;
use std::collections::HashMap;
use std::time::Duration;
use std::time::Instant;

const PLAYX: f32 = 2000.0;
const PLAYY: f32 = 500.0;

/// Message for game server communications
#[derive(Message)]
pub struct Message(pub String);

/// New game session is created
#[derive(Message)]
#[rtype(usize)]
pub struct Connect {
    pub addr: Recipient<Message>,
}

/// Session is disconnected
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

/// `GameServer` responsible for coordinating game sessions.
/// implementation is super primitive
pub struct GameServer {
    sessions: HashMap<usize, Recipient<Message>>,
    players: HashMap<usize, Player>,
    bullets: HashMap<usize, Vec<Bullet>>,
    demon: Demon,
    rng: RefCell<ThreadRng>,
}

#[derive(Serialize)]
struct ClientPlayer {
    pos: Vector2<f32>,
    anchor: Vector2<f32>,
    name: String,
    angle: f32,
    health: u8,
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
            vel: Vector2::new(1.0, 0.0),
            health: 255,
        };
        GameServer {
            sessions: HashMap::new(),
            players: HashMap::new(),
            bullets: HashMap::new(),
            demon,
            rng: RefCell::new(rand::thread_rng()),
        }
    }
    /// Send message to all players
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
                d.health = d.health.saturating_add(1);
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

                if p.shielding {
                    let ox = p.angle.cos();
                    let oy = p.angle.sin();
                    p.anchor.x = p.pos.x + ox * 200.0;
                    p.anchor.y = p.pos.y + oy * 200.0;
                }
                if p.shooting && p.last_shot.elapsed().as_millis() > 500 {
                    let recoil = Vector2::new(p.angle.cos(), p.angle.sin()) * 8.0;
                    let bullet = Bullet {
                        pos: p.pos,
                        vel: recoil,
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

            for bg in act.bullets.values_mut() {
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
                    if b.pos.x < act.demon.pos.x + 25.0
                        && b.pos.x > act.demon.pos.x - 25.0
                        && b.pos.y < act.demon.pos.y + 50.0
                        && b.pos.y > act.demon.pos.y - 50.0
                    {
                        b.time -= Duration::from_secs(3);
                        act.demon.health = act.demon.health.saturating_sub(100);
                    }
                }

                bg.retain(|b| b.time.elapsed().as_millis() < 3000)
            }

            act.reap_players();

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

/// Make actor from `GameServer`
impl Actor for GameServer {
    /// We are going to use simple Context, we just need ability to communicate
    /// with other actors.
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
                ClientMessage::Shoot(s) => p.shooting = s, //500
                ClientMessage::Shield(s) if !s && p.shielding => {
                    p.anchor.x = p.pos.x + p.angle.cos() * 400.0;
                    p.anchor.y = p.pos.y + p.angle.sin() * 400.0;
                    p.last_shield = Instant::now();
                    p.shielding = false;
                } //400
                ClientMessage::Shield(s) => {
                    if p.last_shield.elapsed().as_millis() > 400 {
                        p.shielding = s
                    }
                } //400
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

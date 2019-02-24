use ::actix::*;
use actix_web::server::HttpServer;
use actix_web::*;
use na::Vector2;
use nalgebra as na;
use serde_json::json;

mod server;
use server::{ClientMessage, Connect, Disconnect, GameServer, Message, ServerMessage};

struct WsGameSessionState {
    addr: Addr<GameServer>,
}

fn game_route(req: &HttpRequest<WsGameSessionState>) -> Result<HttpResponse> {
    ws::start(req, WsGameSession { id: 0 })
}

struct WsGameSession {
    id: usize,
}

impl Actor for WsGameSession {
    type Context = ws::WebsocketContext<Self, WsGameSessionState>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let addr: Addr<_> = ctx.address();
        ctx.state()
            .addr
            .send(Connect {
                addr: addr.recipient(),
            })
            .into_actor(self)
            .then(|res, act, ctx| {
                match res {
                    Ok(res) => {
                        act.id = res;
                        ctx.text(
                            json!({
                                "you": act.id.to_string(),
                                "pos": Vector2::new(400.0, 400.0),
                            })
                            .to_string(),
                        );
                    }
                    _ => ctx.stop(),
                }
                fut::ok(())
            })
            .wait(ctx);
    }

    fn stopping(&mut self, ctx: &mut Self::Context) -> Running {
        ctx.state().addr.do_send(Disconnect { id: self.id });
        Running::Stop
    }
}

impl Handler<Message> for WsGameSession {
    type Result = ();

    fn handle(&mut self, msg: Message, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}

impl StreamHandler<ws::Message, ws::ProtocolError> for WsGameSession {
    fn handle(&mut self, msg: ws::Message, ctx: &mut Self::Context) {
        match msg {
            ws::Message::Ping(msg) => ctx.pong(&msg),
            ws::Message::Pong(_) => println!("Ping"),
            ws::Message::Text(text) => {
                let m: ClientMessage = serde_json::from_str(text.trim()).unwrap();
                ctx.state().addr.do_send(ServerMessage { id: self.id, m });
            }
            ws::Message::Binary(_) => println!("Unexpected binary"),
            ws::Message::Close(_) => {
                ctx.stop();
            }
        }
    }
}

fn main() {
    let port = std::env::var("PORT").unwrap_or("8080".to_string());
    let sys = actix::System::new("swingy");

    let server: Addr<_> = Arbiter::start(|_| GameServer::new());

    HttpServer::new(move || {
        let state = WsGameSessionState {
            addr: server.clone(),
        };

        App::with_state(state)
            .resource("/ws/", |r| r.f(game_route))
            .handler(
                "/",
                fs::StaticFiles::new("static/")
                    .unwrap()
                    .index_file("index.html"),
            )
    })
    .bind(format!("0.0.0.0:{}", port))
    .unwrap()
    .start();

    println!("Started http server: http://localhost:{}", port);
    let _ = sys.run();
}

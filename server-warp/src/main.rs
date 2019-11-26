use env_logger::{self, Env};
#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

use futures::{future, Future, FutureExt, StreamExt};
use futures_sink::Sink;
use tokio::sync::mpsc;
use warp::ws::{Message, WebSocket};
use warp::Filter;

use std::collections::HashMap;
use std::env;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex,
};

const LOG_VAR: &str = "SIGNALER_LOG";
const BIND_VAR: &str = "SIGNALER_BIND";
const BIND_TO: &str = "127.0.0.1:8080";

type ClientSessions = Arc<Mutex<HashMap<usize, mpsc::UnboundedSender<Result<Message, warp::Error>>>>>;

static NEXT_SESSION_ID: AtomicUsize = AtomicUsize::new(1);
fn session_connected(ws: WebSocket, client_sessions: ClientSessions) -> impl Future<Output = Result<(), ()>> {
    // Use a counter to assign a new unique ID for this user.
     let my_id = NEXT_SESSION_ID.fetch_add(1, Ordering::Relaxed);

     eprintln!("new chat user: {}", my_id);
 
     // Split the socket into a sender and receive of messages.
     let (user_ws_tx, user_ws_rx) = ws.split();
 
     // Use an unbounded channel to handle buffering and flushing of messages
     // to the websocket...
     let (tx, rx) = mpsc::unbounded_channel();
     warp::spawn(
         rx
             .forward(user_ws_tx)
             .map(|result| {
                 if let Err(e) = result {
                     eprintln!("websocket send error: {}", e);
                 }
             })
     );
 
     // Save the sender in our list of connected client_sessions.
     client_sessions.lock().unwrap().insert(my_id, tx);
 
     // Return a `Future` that is basically a state machine managing
     // this specific user's connection.
 
     // Make an extra clone to give to our disconnection handler...
     let sessions_clone = client_sessions.clone();
 
     match user_ws_tx.start_send(Ok(Message::text("test"))) {
        Ok(()) => (),
        Err(_disconnected) => {
            // The tx is disconnected, our `user_disconnected` code
            // should be happening in another task, nothing more to
            // do here.
        }
    }
     user_ws_rx
         // Every time the user sends a message, broadcast it to
         // all other users...
         .for_each(move |msg| {
             // user_message(my_id, msg.unwrap(), &users);
             trace!("received {:?}", msg);
             future::ready(())
         })
         // for_each will keep processing as long as the user stays
         // connected. Once they disconnect, then...
         .then(move |result| {
             session_disconnected(my_id, &sessions_clone);
             trace!("disconnected {}", my_id);
             future::ok(result)
         })
         // If at any time, there was a websocket error, log here...
         // .map_err(move |e| {
         //     eprintln!("websocket error(uid={}): {}", my_id, e);
         // })
}
fn session_disconnected(my_id: usize, sessions: &ClientSessions) {
    debug!("good bye user: {}", my_id);

    // Stream closed up, so remove from the user list
    sessions.lock().unwrap().remove(&my_id);
}

#[tokio::main]
async fn main() {
    color_backtrace::install();
    if env::var(LOG_VAR).is_err() {
        env::set_var(LOG_VAR, "server_warp=trace,info");
    }
    env_logger::init_from_env(Env::new().filter(LOG_VAR));
    let bind_to = env::var(BIND_VAR).unwrap_or_else(|_| BIND_TO.into());

    info!("listening on http://{}", bind_to);
    debug!("debug");
    trace!("trace");

    let client_sessions = Arc::new(Mutex::new(HashMap::new()));
    // Turn our "state" into a new Filter...
    let client_sessions = warp::any().map(move || client_sessions.clone());

    let index = warp::get().and(warp::fs::dir("../static/"));

    let ws = warp::path("ws")
        // The `ws()` filter will prepare the Websocket handshake.
        .and(warp::ws())
        .and(client_sessions)
        .map(|ws: warp::ws::Ws, client_sessions| {
            // And then our closure will be called when it completes...
            ws.on_upgrade(move |socket| session_connected(socket, client_sessions).map(|result| result.unwrap()))
        });
    let routes = index.or(ws);

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}

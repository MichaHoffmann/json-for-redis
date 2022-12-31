extern crate redis;

use rand::{distributions::Alphanumeric, distributions::Uniform, Rng};
use std::env;
use std::iter::repeat;
use std::net::TcpListener;
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::{thread, time};
use test_context::TestContext;

pub struct Ctx {
    redis: Child,
    client: redis::Client,
}

impl Ctx {
    pub fn connection(&mut self) -> redis::Connection {
        self.client
            .get_connection()
            .expect("failed to get connection")
    }
}

impl TestContext for Ctx {
    fn setup() -> Ctx {
        let port = random_port();
        let module = env::var("REDIS_JSON_MODULE").expect("REDIS_JSON_MODULE not set");
        let ctx = Ctx {
            redis: Command::new("redis-server")
                .arg("--save \"\"")
                .arg(format!("--port {port}"))
                .arg(format!("--loadmodule {module}"))
                .stdout(Stdio::null())
                .spawn()
                .expect("failed to start redis"),
            client: redis::Client::open(format!("redis://0.0.0.0:{port}/"))
                .expect("failed to create client"),
        };

        repeat(())
            .take(10)
            .find(|_| {
                thread::sleep(time::Duration::from_millis(100));
                ctx.client.get_connection().is_ok()
            })
            .expect("failed to connect to redis");

        ctx
    }

    fn teardown(mut self) {
        self.redis.kill().expect("killing redis failed");
        self.redis.wait().expect("waiting redis failed");
    }
}

fn random_port() -> u16 {
    static MUX: Mutex<i32> = Mutex::new(0);
    let _lock = MUX.lock().expect("unable to lock port selection");

    rand::thread_rng()
        .sample_iter(Uniform::new(1024, 49151))
        .find(|port| TcpListener::bind(("127.0.0.1", *port)).is_ok())
        .unwrap()
}

pub fn random_key(size: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(size)
        .map(char::from)
        .collect()
}

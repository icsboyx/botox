mod defs;

mod bus;
use std::sync::Arc;

use tokio::time::{ sleep, Duration };

use bus::*;
use futures::{ future::maybe_done, FutureExt };
use tokio::{ io::AsyncBufReadExt, pin, task };

#[tokio::main]
async fn main() {
    let bus = Bus::new();

    let mut tasks = Vec::new();

    for i in 0..1000 {
        tasks.push(task(bus.clone(), i).fuse());
    }

    for task in tasks {
        tokio::spawn(task);
    }

    tokio::spawn(task_user_input(bus.clone()));

    loop {
        sleep(Duration::from_secs(1)).await;
    }

    // pin! {
    //     let task_01 = task(&bus, 1).fuse();
    //     let task_02 = task(&bus, 2).fuse();
    //     let task_user_input = task_user_input(&bus).fuse();
    // }

    // loop {
    //     tokio::select! {
    //             _ = &mut task_01 => {
    //                 println!("[MAIN] task_01 finished");
    //             }
    //             _ = &mut task_02 => {
    //                 println!("[MAIN] task_02 finished");
    //            }
    //             _ = &mut task_user_input => {
    //                 println!("[MAIN] task_user_input finished");
    //         }
    //     }
    // }
}

async fn task(bus: Arc<Bus>, task_number: i32) {
    let my_id = format!("task_{:02}", task_number);
    println!("[{}] Start task.", &my_id);
    let my_subscriber = bus.register(&my_id).await;
    bus.subscribe("main", my_subscriber.clone()).await;

    loop {
        my_subscriber.queue().event().notified().await;
        println!("[{}] Queue: {:?}", &my_id, my_subscriber.queue().display().await);
        if let Some(message) = my_subscriber.queue().pop_front().await {
            println!("[{}] Event received: {}", &my_id, message);
        }
    }
}

async fn task_user_input(queue: Arc<Bus>) {
    // read user input
    let mut stdin = tokio::io::BufReader::new(tokio::io::stdin());
    println!("[USER] Start task_user_input");
    let my_subscriber = queue.register("main").await;
    loop {
        let mut line = String::new();
        stdin.read_line(&mut line).await.unwrap();
        let line = line.trim().to_owned();
        println!("[USER] sending: {:#?}", line.trim());
        my_subscriber.send(line).await;
    }
}

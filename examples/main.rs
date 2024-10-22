async fn bottom() -> u32 {
    println!("bottom");
    7
}

async fn middle() -> u32 {
    println!("middle");
    bottom().await
}
async fn top() -> u32 {
    println!("top");
    middle().await
}

#[oxic::main]
async fn entry() {
    let i = top().await;
    println!("{}", i);
}

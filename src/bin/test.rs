fn main() {
    async_std::task::block_on((|| async {
        let manager = gcp_auth::init().await.unwrap();
        println!(
            "access token: {}",
            manager.get_token(&[]).await.unwrap().as_str()
        );
    })())
}

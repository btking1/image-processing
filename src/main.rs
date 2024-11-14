use chill::Chill;

mod chill;

// fn chunks(amount: u8) {
//     let vector = vec!["something"; 8];
//     let mut chunks = Vec::new();
//     for _ in 0..=amount {
//         let chunk = vector.chunks(chunk_size)
//     }
//     println!("{:?}", chunks)
// }
fn main() {
    let chill = chill::Chill {
        //pexel api key
        api_key: "563492ad6f917000010000016271bb6cc0614b44bf7d12ba3a610eaa".to_string(),
        user: "godsiom".to_string(),
        browser: "firefox".to_string(),
        page: 1,
        photo: 1,
        per_page: 2,
    };
    let make_gif = Chill::gif(&chill);
    make_gif.expect("ERROR -> problem making gif")
    // Chill::display(&chill).unwrap();
}

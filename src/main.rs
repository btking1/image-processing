mod chill;

fn main() {
    let chill = chill::Chill {
        //pexel api key
        api_key: "".to_string(),
        user: "godsiom".to_string(),
        browser: "firefox".to_string(),
        page: 1,
        photo: 1,
        per_page: 2,
    };

    // Chill::display(&chill).unwrap();
}

use icu_experimental::transliterate::Transliterator;

fn main() {
    assert_eq!(
        Transliterator::try_new(&"und-Hira-t-und-kana".parse().unwrap())
            .unwrap()
            .transliterate("ウィキペディア".into()),
        "うぃきぺでぃあ"
    );

    assert_eq!(
        Transliterator::try_new(&"und-Kana-t-und-hira".parse().unwrap())
            .unwrap()
            .transliterate("うぃきぺでぃあ".into()),
        "ウィキペディア"
    );

    assert_eq!(
        dbg!(Transliterator::try_new(&"und-x-autocomp".parse().unwrap()).unwrap())
            .transliterate("Täst 😒 Ω".into()),
        "tast 😒 ω"
    );

    assert_eq!(
        Transliterator::try_new(&"de-x-autocomp".parse().unwrap())
            .unwrap()
            .transliterate("Täst 😒 Ω".into()),
        "taest 😒 ω"
    );

    Transliterator::chain([
        Transliterator::new_lower(),
        Transliterator::nfd(),
        Transliterator::remove(|c| "-~֊־᠆‐‑‒–—―⁓⁻₋−⸺⸻〜〰゠﹘﹣－".contains(c)),
        Transliterator::nfc(),
        Transliterator::new_latin_ascii(),
    ])
}

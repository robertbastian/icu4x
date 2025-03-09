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
        Transliterator::try_new(&"und-x-autocomp".parse().unwrap())
            .unwrap()
            .transliterate("Täst 😒 Ω".into()),
        "tast 😒 ω"
    );

    assert_eq!(
        Transliterator::try_new(&"de-x-autocomp".parse().unwrap())
            .unwrap()
            .transliterate("Täst 😒 Ω".into()),
        "taest 😒 ω"
    );
}

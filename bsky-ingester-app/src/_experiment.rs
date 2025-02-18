// Only English. We'll skip posts in other languages.
static STOP_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| get(LANGUAGE::English));
static PUNCTUATION: LazyLock<Vec<String>> = LazyLock::new(|| {
    [
        ".", ",", ":", ";", "!", "?", "(", ")", "[", "]", "{", "}", "\"", "'", "-", "_", "+", "=",
        "*", "/", "\\", "|", "<", ">", "~", "`", "^", "$", "%", "@", "&", "#",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect()
});

static COMMON_WORDS: LazyLock<BTreeSet<String>> = LazyLock::new(|| {
    let file = File::open("data/words.en.txt").unwrap();
    let mut reader = BufReader::new(file);
    reader
        .lines()
        .filter_map(|line| {
            let line = line.ok()?;
            let line = line.trim();
            if line.is_empty() {
                None
            } else {
                Some(line.to_string())
            }
        })
        .collect()
});

#[cfg(test)]
mod tests {
    use super::*;

    use keyword_extraction::tf_idf::{TfIdf, TfIdfParams};

    #[test]
    fn it_works() {
        let documents: Vec<String> = vec![
            "Let's write some Rust.".to_string(),
            "Rust is a language.".to_string(),
            "I wrote some CUDA code.".to_string(),
            "CUDA is a parallel computing platform and programming model.".to_string(),
            "Can you write CUDA in Rust?".to_string(),
            "I can write CUDA in Rust.".to_string(),
            "Anyone using React.js?".to_string(),
        ];

        let mut sw = (*STOP_WORDS).clone();
        sw.extend((*COMMON_WORDS).iter().cloned());

        for doc in documents.iter() {
            let yake = Yake::new(YakeParams::All(
                doc,
                // &*STOP_WORDS,
                &sw,
                Some(&*PUNCTUATION),
                0.9,
                3,
                1,
            ));
            dbg!(yake.get_ranked_keyword_scores(5));
        }

        // let params =
        //     TfIdfParams::UnprocessedDocuments(&documents, &*STOP_WORDS, Some(&*PUNCTUATION));
        // let tf_idf = TfIdf::new(params);
        // dbg!(tf_idf.get_ranked_word_scores(5));
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    {
        // let mut sw = (*STOP_WORDS).clone();
        // sw.extend((*COMMON_WORDS).iter().cloned());

        // let yake = Yake::new(YakeParams::All(
        //     &post,
        //     // &*STOP_WORDS,
        //     &sw,
        //     Some(&*PUNCTUATION),
        //     0.9,
        //     3,
        //     1,
        // ));

        // let messages: Vec<_> = yake
        //     .get_ranked_keyword_scores(10)
        //     .into_iter()
        //     .map(|(keyword, _)| serde_json::to_vec(&KeywordMessage { value: keyword }).unwrap())
        //     .collect();
    }
    {

        // let yake = Yake::new(YakeParams::All(
        //     &post.text,
        //     // &*STOP_WORDS,
        //     &sw,
        //     Some(&*PUNCTUATION),
        //     0.9,
        //     3,
        //     1,
        // ));

        // let messages: Vec<_> = yake
        //     .get_ranked_keyword_scores(10)
        //     .into_iter()
        //     .map(|(keyword, _)| serde_json::to_vec(&KeywordMessage { value: keyword }).unwrap())
        //     .chain(
        //         hashtag_regex
        //             .captures_iter(&post.text)
        //             .filter_map(|c| c.name("tag").map(|m|
        //                 //  m.as_str().as_bytes().to_vec()
        //                 serde_json::to_vec(&KeywordMessage { value: m.as_str().into() }).unwrap()
        //         )),
        //     )
        //     .collect();

        // broker
        //     .publish_batch(KEYWORDS_BROKER_SUBJECT, messages)
        //     .await?;
    }
    Ok(())
}

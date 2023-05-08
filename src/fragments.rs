use simplelog::*;
use tiktoken_rs::p50k_base;

// Any higher can be truncated by the encoding model
const MAX_TOKENS: usize = 256;

pub fn tokenizer(text: &str) -> Vec<&str>  {
    text.split(' ').collect::<Vec<&str>>() 
}

pub fn _tokenize(text: &str) -> Vec<usize> {
  let bpe = p50k_base().unwrap();
  let tokens = bpe.encode_with_special_tokens(text);
  dbg!(&tokens);
  tokens
}

pub fn _decode(tokens: Vec<usize>) -> String {
  let bpe = p50k_base().unwrap();
  let text = bpe.decode(tokens);
  text.unwrap()
}

pub fn create_fragments_from_text(document: String, settings: &config::Config) -> Vec<String>  {
  let tokens = tokenizer(&document);

  let mut max_tokens = 0;
  if let Ok(max) = settings.get_int("database.max_tokens") {
    max_tokens = max as usize;
  }
  
  if max_tokens == 0 || max_tokens > MAX_TOKENS { // Prevent truncation
    max_tokens = MAX_TOKENS as usize; 
  }

  let mut fragments: Vec<String> = Vec::new();
  let mut fragment = Vec::new();
  let mut fragment_length = 0;
  let token_total = tokens.clone().len();

  info!("Token total: {}", token_total);
  info!("Max tokens: {}", max_tokens);

  for token in tokens {
    fragment.push(token.clone());
    fragment_length += 1;

    if fragment_length == max_tokens || token_total == fragment_length {
      fragments.push(fragment.join(" "));
      fragment = Vec::new();
      fragment_length = 0;
    }
  }

  fragments
}

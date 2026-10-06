use llama_cpp_2::{
    context::params::LlamaContextParams,
    llama_backend::LlamaBackend,
    llama_batch::LlamaBatch,
    model::{params::LlamaModelParams, LlamaModel},
    sampling::LlamaSampler,
};

use std::num::NonZeroU32;

pub struct LlmEngine {
    model: LlamaModel,
}

impl LlmEngine {
    pub fn load(backend: &LlamaBackend, model_path: &str) -> Result<Self, String> {
        let model_params = LlamaModelParams::default();

        let model = LlamaModel::load_from_file(backend, model_path, &model_params)
            .map_err(|e| e.to_string())?;

        Ok(Self { model })
    }

    pub fn generate(&mut self, backend: &LlamaBackend, prompt: &str) -> Result<String, String> {
        let stop_sequences = ["<END>", "\nUser:", "<|im_end|>"];
        let ctx_params = LlamaContextParams::default()
            .with_n_ctx(NonZeroU32::new(32000))
            .with_n_batch(512)
            .with_n_ubatch(512);

        let mut ctx = self
            .model
            .new_context(&backend, ctx_params)
            .map_err(|e| e.to_string())?;

        let vocab = self.model.vocab();

        let tokens = vocab.tokenize(prompt.as_bytes(), true, true);

        println!("Prompt tokens: {}", tokens.len());

        let mut batch = LlamaBatch::new(512, 1);

        let batch_size = 512usize;

        for (chunk_index, chunk) in tokens.chunks(batch_size).enumerate() {
            batch.clear();

            let start = chunk_index * batch_size;

            for (i, token) in chunk.iter().enumerate() {
                let position = start + i;
                let is_last = position == tokens.len() - 1;
                batch
                    .add(*token, position as i32, &[0], is_last)
                    .map_err(|e| e.to_string())?;
            }

            ctx.decode(&mut batch).map_err(|e| e.to_string())?;
        }

        let mut sampler =
            LlamaSampler::chain_simple([LlamaSampler::temp(0.7), LlamaSampler::dist(1234)]);

        let mut output = String::new();

        let mut position = tokens.len() as i32;

        for _ in 0..500 {
            let token = sampler.sample(&ctx, batch.n_tokens() - 1);

            sampler.accept(token);

            if vocab.is_eog(token) {
                break;
            }

            let bytes = vocab.token_to_piece(token, true, None);

            let text = String::from_utf8_lossy(&bytes);
            output.push_str(&text);

            if let Some(stop) = stop_sequences.iter().find(|stop| output.ends_with(**stop)) {
                output.truncate(output.len() - stop.len());
                break;
            }

            batch.clear();

            batch
                .add(token, position, &[0], true)
                .map_err(|e| e.to_string())?;

            ctx.decode(&mut batch).map_err(|e| e.to_string())?;

            position += 1;
        }

        Ok(output)
    }
}

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
        // 3. Create context
        let ctx_params =
            LlamaContextParams::default().with_n_ctx(Some(NonZeroU32::new(2048).unwrap()));

        let mut ctx = self
            .model
            .new_context(&backend, ctx_params)
            .map_err(|e| e.to_string())?;

        let vocab = self.model.vocab();

        let tokens = vocab.tokenize(
            prompt.as_bytes(),
            true, // add_special
            true, // parse_special
        );

        // 5. Create initial batch
        let mut batch = LlamaBatch::new(512, 1);

        let last_index = tokens.len() - 1;

        for (i, token) in tokens.iter().enumerate() {
            batch
                .add(*token, i as i32, &[0], i == last_index)
                .map_err(|e| e.to_string())?;
        }

        // 6. Feed prompt into model
        ctx.decode(&mut batch).map_err(|e| e.to_string())?;

        // 7. Configure sampling
        let mut sampler =
            LlamaSampler::chain_simple([LlamaSampler::temp(0.7), LlamaSampler::dist(1234)]);

        let mut output = String::new();

        let mut position = tokens.len() as i32;

        // 8. Generate up to 200 tokens
        for _ in 0..500 {
            let token = sampler.sample(&ctx, batch.n_tokens() - 1);

            sampler.accept(token);

            // Stop when model emits EOS/EOG
            if vocab.is_eog(token) {
                break;
            }

            // Convert token -> text
            let bytes = vocab.token_to_piece(token, true, None);

            let text = String::from_utf8_lossy(&bytes);
            output.push_str(&text);

            // Feed generated token back into model
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

use super::Context;
use crate::model::Model;

use anyhow::Result;

pub struct Master {
    ctx: Context,
    model: Model,
}

impl Master {
    pub async fn new(ctx: Context) -> Result<Self> {
        let model = Model::load(ctx.clone()).await?;

        Ok(Self { ctx, model })
    }

    pub async fn generate<S>(&mut self, stream: S) -> Result<()>
    where
        S: Fn(&str),
    {
        log::info!(
            "starting the inference loop (mem={})\n\n",
            human_bytes::human_bytes(memory_stats::memory_stats().unwrap().physical_mem as f64)
        );

        log::debug!("  ctx.args.sample_len = {}", self.ctx.args.sample_len);

        stream(&self.ctx.args.prompt);

        let mut start_gen = std::time::Instant::now();

        for index in 0..self.ctx.args.sample_len {
            if index == 1 {
                // record start time again since the first token is the warmup
                start_gen = std::time::Instant::now()
            }

            let token = self.model.next_token(index).await?;
            if token.is_eos {
                break;
            } else {
                stream(&token.to_string());
            }
        }

        if let Some(rest) = self.model.final_token()? {
            stream(&rest);
        }

        // signal end of stream
        stream("");

        let dt = start_gen.elapsed();
        let generated = self.model.generated_tokens();

        log::info!(
            "{} tokens generated ({} token/s) - mem={}",
            generated,
            (generated - 1) as f64 / dt.as_secs_f64(),
            human_bytes::human_bytes(memory_stats::memory_stats().unwrap().physical_mem as f64)
        );

        Ok(())
    }
}

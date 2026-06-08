use crate::agent::tools;
use crate::extras::subagents::prompt;
use crate::provider::{AnyAgent, AnyModel, OpenAiAgent, OpenAiModel};
use rig::agent::{Agent, AgentBuilder};
use rig::completion::CompletionModel;

fn build_explore_agent_inner<M: CompletionModel + 'static>(
    model: M,
    max_turns: usize,
    max_text_file_size: u64,
    // OpenRouter `provider.order` pin for `anthropic/*` (see `AnyClient::completion_model`).
    additional_params: Option<serde_json::Value>,
    #[cfg(feature = "archmd")] architecture: Option<&str>,
) -> Agent<M> {
    let mut preamble = prompt::explore_prompt();

    #[cfg(feature = "archmd")]
    if let Some(arch) = architecture
        && !arch.is_empty()
    {
        preamble.push_str("\n\n");
        preamble.push_str(arch);
    }

    // Subagents use the same built-in defaults as the main agent's
    // `resolve_*` methods (2000 lines, 200 results, no list_dir cap) so a
    // user who hasn't customized config sees identical subagent behaviour.
    // Plumbing the user's resolved config through to subagents could be a
    // future change if local-LLM operators want their tight limits to
    // apply to subagent calls too.
    let tools: Vec<Box<dyn rig::tool::ToolDyn>> = vec![
        Box::new(tools::ReadTool::new(
            None,
            None,
            Some(max_text_file_size),
            2000,
        )),
        Box::new(tools::GrepTool::new(None, None, 200)),
        Box::new(tools::FindFilesTool::new(None, None, 200)),
        Box::new(tools::ListDirTool::new(None, None, None)),
        #[cfg(feature = "memory")]
        Box::new(crate::extras::memory::MemoryRead::new(None, None)),
        #[cfg(feature = "memory")]
        Box::new(crate::extras::memory::MemorySearch::new(None, None)),
    ];

    let mut builder = AgentBuilder::new(model)
        .preamble(&preamble)
        .default_max_turns(max_turns)
        .tools(tools);

    if let Some(params) = additional_params {
        builder = builder.additional_params(params);
    }

    builder.build()
}

pub(crate) async fn build_explore_agent(
    model: AnyModel,
    max_turns: usize,
    #[cfg(feature = "archmd")] architecture: Option<String>,
) -> AnyAgent {
    // Use a reasonable default file size for subagent reads
    let max_text_file_size = 10 * 1024 * 1024;
    #[cfg(feature = "archmd")]
    let arch_ref = architecture.as_deref();
    match model {
        AnyModel::OpenRouter(m, extra) => AnyAgent::OpenRouter(build_explore_agent_inner(
            m,
            max_turns,
            max_text_file_size,
            extra,
            #[cfg(feature = "archmd")]
            arch_ref,
        )),
        AnyModel::OpenAI(m) => AnyAgent::OpenAI(match m {
            OpenAiModel::Responses(m) => OpenAiAgent::Responses(build_explore_agent_inner(
                m,
                max_turns,
                max_text_file_size,
                None,
                #[cfg(feature = "archmd")]
                arch_ref,
            )),
            OpenAiModel::Completions(m) => OpenAiAgent::Completions(build_explore_agent_inner(
                m,
                max_turns,
                max_text_file_size,
                None,
                #[cfg(feature = "archmd")]
                arch_ref,
            )),
        }),
        AnyModel::Anthropic(m) => AnyAgent::Anthropic(build_explore_agent_inner(
            m,
            max_turns,
            max_text_file_size,
            None,
            #[cfg(feature = "archmd")]
            arch_ref,
        )),
        AnyModel::Gemini(m) => AnyAgent::Gemini(build_explore_agent_inner(
            m,
            max_turns,
            max_text_file_size,
            None,
            #[cfg(feature = "archmd")]
            arch_ref,
        )),
        AnyModel::Ollama(m) => AnyAgent::Ollama(build_explore_agent_inner(
            m,
            max_turns,
            max_text_file_size,
            None,
            #[cfg(feature = "archmd")]
            arch_ref,
        )),
    }
}

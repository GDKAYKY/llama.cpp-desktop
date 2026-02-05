<script lang="ts">
  import {
    AnthropicLogo,
    ClaudeAILogo,
    MistralAILogo,
    MetaLogo,
    OpenAILogo,
    NVIDIALogo,
    GoogleLogo,
    GeminiLogo,
    StabilityAILogo,
    CohereLogo,
    HuggingFaceLogo,
    DeepSeekLogo,
    GrokLogo,
    QwenLogo,
    MicrosoftLogo,
    OllamaLogo,
  } from "@selemondev/svgl-svelte";
  import * as SimpleIcons from "@icons-pack/svelte-simple-icons";
  import { Box, Sparkles, Bot } from "lucide-svelte";

  interface Props {
    name: string;
    size?: number;
    class?: string;
  }

  let { name, size = 24, class: className = "" }: Props = $props();

  /**
   * Mapping for SVGL icons (Higher quality brand logos)
   */
  const svglIcons: Record<string, any> = {
    anthropic: AnthropicLogo,
    claude: ClaudeAILogo,
    mistral: MistralAILogo,
    meta: MetaLogo,
    llama: MetaLogo,
    openai: OpenAILogo,
    gpt: OpenAILogo,
    nvidia: NVIDIALogo,
    google: GoogleLogo,
    gemini: GeminiLogo,
    stability: StabilityAILogo,
    cohere: CohereLogo,
    huggingface: HuggingFaceLogo,
    hf: HuggingFaceLogo,
    deepseek: DeepSeekLogo,
    qwen: QwenLogo,
    grok: GrokLogo,
    xai: GrokLogo,
    microsoft: MicrosoftLogo,
    azure: MicrosoftLogo,
    ollama: OllamaLogo,
    ai: Sparkles,
  };

  /**
   * Helper to find the best matching icon with fallback logic
   */
  function getIcon(name: string) {
    const lowerName = name.toLowerCase();

    // 1. Try SVGL (Primary for AI brands)
    for (const [key, icon] of Object.entries(svglIcons)) {
      if (lowerName.includes(key)) {
        return icon;
      }
    }

    // 2. Try Simple Icons (Secondary)
    // We try to match common slugs
    const SI = SimpleIcons as any;
    const brandMatch = lowerName.split(/[^a-z0-9]/)[0]; // Get first word
    const capitalizedMatch =
      "Si" + brandMatch.charAt(0).toUpperCase() + brandMatch.slice(1);

    if (SI[capitalizedMatch]) {
      return SI[capitalizedMatch];
    }

    // 3. Last Resort: Generic Bot or Sparkles
    return Bot;
  }

  const Icon = $derived(getIcon(name));
</script>

<div
  class="flex items-center justify-center shrink-0 overflow-hidden"
  style="width: {size}px; height: {size}px;"
>
  <Icon {size} width={size} height={size} class={className} />
</div>

<style>
  div :global(svg) {
    width: 100% !important;
    height: 100% !important;
    max-width: 100%;
    max-height: 100%;
  }
</style>

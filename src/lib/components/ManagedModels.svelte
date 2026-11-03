<script lang="ts">
  import type { ModelTerms, ProtocolProviderStatus } from '../workflow/types';
  import { classDisclosure, memoryClasses, modelLadder } from '../models/modelLadder';
  import { t } from '../i18n';
  import { errorMessage } from '../errors';
  import ModelClass from './ModelClass.svelte';

  export let terms: ModelTerms[];
  export let status: ProtocolProviderStatus;
  export let providerStatus: ProtocolProviderStatus;
  export let memoryGb: number | null;
  export let downloading: Record<string, number>;
  export let onAcceptLicence: (model: string) => Promise<void>;
  export let onDownloadModel: (model: string) => Promise<void>;
  export let onCancelDownload: (model: string) => Promise<void>;
  export let onChoose: (model: string) => Promise<void>;
  export let onChooseProvider: (model: string) => Promise<void>;

  $: ladder = modelLadder(terms, providerStatus.models, memoryGb);
  $: groups = memoryClasses(ladder);
  $: recommendedClass = ladder.find((model) => model.recommended)?.minimumMemoryGb ?? null;
  $: disclosure = classDisclosure(groups, memoryGb, recommendedClass);
  $: selected = status.selectedModel ?? providerStatus.selectedModel;
  $: selectedIsListed =
    selected === null ||
    ladder.some(
      (model) => model.bundled?.modelId === selected || model.provider?.name === selected,
    );
  $: shared = {
    status,
    providerStatus,
    downloading,
    onAcceptLicence,
    onDownloadModel,
    onCancelDownload,
    onChoose,
    onChooseProvider,
  };
</script>

<section aria-label={$t.settings.curatedModels}>
  <p class="model-kicker">{$t.settings.curatedModels}</p>
  <p class="setting-hint">{$t.settings.downloadIntro}</p>
  <p class="setting-hint">
    {memoryGb === null ? $t.settings.memoryUnknown : $t.settings.memoryReported(memoryGb)}
  </p>
  {#if memoryGb !== null && memoryGb <= 8}<p class="setting-hint">{$t.settings.memoryTight}</p>{/if}
  {#if memoryGb !== null && memoryGb < 8}<p class="setting-hint">
      {$t.settings.memoryBelowMinimum}
    </p>{/if}
  <p class="setting-hint">{$t.settings.recommendationNote}</p>
  <p class="setting-hint">{$t.settings.qualityCaution}</p>
  {#if !status.serverReachable}<p class="setting-hint">{errorMessage(status.message)}</p>{/if}
  {#if selected !== null && !selectedIsListed}<p class="setting-hint">
      {$t.settings.selectedModelNotOffered}
    </p>{/if}
  {#each disclosure.own as group (group.minimumMemoryGb)}
    <ModelClass {...shared} {group} />
  {/each}
  {#if disclosure.larger.length}
    <details class="model-disclosure">
      <summary class="quiet-action">{$t.settings.largerModels}</summary>
      {#each disclosure.larger as group (group.minimumMemoryGb)}
        <ModelClass {...shared} {group} />
      {/each}
    </details>
  {/if}
  {#if disclosure.smaller.length}
    <details class="model-disclosure">
      <summary class="quiet-action">{$t.settings.smallerModels}</summary>
      {#each disclosure.smaller as group (group.minimumMemoryGb)}
        <ModelClass {...shared} {group} />
      {/each}
    </details>
  {/if}
</section>

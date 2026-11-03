<script lang="ts">
  import type { ModelTerms, ProtocolProviderStatus } from '../workflow/types';
  import { memoryClasses, modelLadder, withoutModel } from '../models/modelLadder';
  import { t } from '../i18n';
  import { errorMessage } from '../errors';
  import ModelCard from './ModelCard.svelte';
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
  $: recommended = ladder.find((model) => model.recommended) ?? null;
  $: recommendedId = recommended?.entry.id ?? null;
  $: more = withoutModel(groups, recommendedId ?? '');
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
  <p class="setting-hint">{$t.settings.qualityCaution}</p>
  {#if !status.serverReachable}<p class="setting-hint">{errorMessage(status.message)}</p>{/if}

  {#if recommended}
    <section class="recommended-models" aria-label={$t.settings.recommendedForMachine}>
      <p class="model-kicker">{$t.settings.recommendedForMachine}</p>
      <div class="model-catalog">
        <ModelCard option={recommended} {...shared} />
      </div>
    </section>
  {/if}

  {#if selected !== null && !selectedIsListed}<p class="setting-hint">
      {$t.settings.selectedModelNotOffered}
    </p>{/if}

  {#if more.length}
    <p class="model-kicker more-models">{$t.settings.moreModels}</p>
  {/if}
  {#each more as group (group.minimumMemoryGb)}
    <ModelClass {...shared} {group} />
  {/each}
</section>

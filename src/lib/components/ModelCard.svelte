<script lang="ts">
  import type { LadderModel } from '../models/modelLadder';
  import type { ProtocolProviderStatus } from '../workflow/types';
  import { formatModelSize } from '../models/modelSize';
  import { listOf, t } from '../i18n';

  export let option: LadderModel;
  export let status: ProtocolProviderStatus;
  export let providerStatus: ProtocolProviderStatus;
  export let downloading: Record<string, number>;
  export let downloadErrors: Record<string, string> = {};
  export let onAcceptLicence: (model: string) => Promise<void>;
  export let onDownloadModel: (model: string) => Promise<void>;
  export let onCancelDownload: (model: string) => Promise<void>;
  export let onChoose: (model: string) => Promise<void>;
  export let onChooseProvider: (model: string) => Promise<void>;

  $: bundled = option.bundled;
  $: provider = option.provider;
  $: progress = bundled ? downloading[bundled.modelId] : undefined;
  $: downloadError = bundled ? downloadErrors[bundled.modelId] : undefined;
  $: licenceSettled = !bundled || !bundled.acceptanceRequired || bundled.acceptedAtMs !== null;

  function sizeOf(model: LadderModel): string {
    if (model.bundled) return formatModelSize(model.bundled.byteCount);
    if (model.provider) return formatModelSize(model.provider.size);
    if (model.entry.installedGb !== null) {
      return $t.settings.modelSizeInstalled(model.entry.installedGb.toLocaleString($t.locale));
    }
    return model.entry.tier === 'baseline'
      ? $t.settings.modelSizeSmall
      : $t.settings.modelSizeLarger;
  }

  function statusOf(model: LadderModel): string {
    if (model.installed) return $t.settings.modelStatus.installed;
    if (model.source === 'planned') return $t.settings.modelStatus.plannedCandidate;
    return $t.settings.modelStatus.notInstalled;
  }

  function selected(model: LadderModel): boolean {
    if (model.bundled?.installed) {
      return status.active !== false && status.selectedModel === model.bundled.modelId;
    }
    if (model.provider) {
      return (
        providerStatus.active !== false && providerStatus.selectedModel === model.provider.name
      );
    }
    return false;
  }
</script>

<article class="model-card" class:active={selected(option)}>
  <div class="model-card-copy">
    <div class="model-card-heading">
      <h3>{option.entry.name}</h3>
      {#if option.source === 'provider'}<span class="model-badge quiet"
          >{$t.settings.viaOllama}</span
        >{/if}
      {#if option.entry.origin === 'european'}<span class="model-badge quiet"
          >{$t.settings.european}</span
        >{/if}
    </div>
    <p>{$t.settings.modelDescription[option.entry.id]}</p>
    <div class="model-meta">
      <span>{option.entry.family}</span>
      <span>{sizeOf(option)}</span>
      <span>{$t.settings.modelLicence[option.entry.licence]}</span>
      <span>{$t.settings.modelOrigin[option.entry.origin]}</span>
      <span
        >{option.entry.languages
          .slice(0, 3)
          .map((code) => $t.settings.modelLanguage[code])
          .join(' · ')}</span
      >
    </div>
    {#if !option.fits}<p class="setting-hint">
        {$t.settings.memoryTooSmall(option.minimumMemoryGb)}
      </p>{/if}
    {#if downloadError}<p class="setting-error" role="alert">{downloadError}</p>{/if}
    <p class="model-evaluation">
      {option.entry.testedLanguages.length
        ? $t.settings.evaluatedIn(
            listOf(option.entry.testedLanguages.map((code) => $t.settings.modelLanguage[code])),
          )
        : $t.settings.evaluationPending}
    </p>
  </div>
  <div class="model-card-action">
    <span class="model-status">{statusOf(option)}</span>
    {#if progress !== undefined}
      <progress
        max="100"
        value={progress}
        aria-label={$t.settings.downloadModel(formatModelSize(bundled?.byteCount ?? 0))}
      ></progress>
      <span>{progress}%</span>
      <button class="quiet-action" onclick={() => onCancelDownload(bundled?.modelId ?? '')}
        >{$t.settings.cancel}</button
      >
    {:else if bundled?.installed}
      <button
        class="secondary-action"
        disabled={selected(option) || !option.fits || !status.serverReachable}
        onclick={() => onChoose(bundled.modelId)}
        >{selected(option) ? $t.settings.modelSelected : $t.settings.useThisModel}</button
      >
    {:else if provider}
      <button
        class="secondary-action"
        disabled={selected(option) || !option.fits || !providerStatus.serverReachable}
        onclick={() => onChooseProvider(provider.name)}
        >{selected(option) ? $t.settings.modelSelected : $t.settings.useThisModel}</button
      >
    {:else if bundled}
      {#if !licenceSettled && option.fits}
        <p>{$t.settings.licenceBody($t.settings.modelLicence[option.entry.licence])}</p>
        <a class="licence-link" href={bundled.licenceUrl} target="_blank" rel="noreferrer noopener"
          >{$t.settings.licenceRead}</a
        >
        <button class="secondary-action" onclick={() => onAcceptLicence(bundled.modelId)}
          >{$t.settings.licenceAccept}</button
        >
      {:else}
        <button
          class="secondary-action"
          disabled={!option.fits}
          onclick={() => onDownloadModel(bundled.modelId)}
          >{$t.settings.downloadModel(formatModelSize(bundled.byteCount))}</button
        >
      {/if}
    {/if}
  </div>
</article>

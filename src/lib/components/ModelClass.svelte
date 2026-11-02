<script lang="ts">
  import type { MemoryClass, LadderModel } from '../models/modelLadder';
  import type { ProtocolProviderStatus } from '../workflow/types';
  import { formatModelSize } from '../models/modelSize';
  import { listOf, t } from '../i18n';

  export let group: MemoryClass;
  export let status: ProtocolProviderStatus;
  export let providerStatus: ProtocolProviderStatus;
  export let downloading: Record<string, number>;
  export let onAcceptLicence: (model: string) => Promise<void>;
  export let onDownloadModel: (model: string) => Promise<void>;
  export let onCancelDownload: (model: string) => Promise<void>;
  export let onChoose: (model: string) => Promise<void>;
  export let onChooseProvider: (model: string) => Promise<void>;

  function sizeOf(option: LadderModel): string {
    if (option.bundled) return formatModelSize(option.bundled.byteCount);
    if (option.provider) return formatModelSize(option.provider.size);
    if (option.entry.installedGb !== null) {
      return $t.settings.modelSizeInstalled(option.entry.installedGb.toLocaleString($t.locale));
    }
    return option.entry.tier === 'baseline'
      ? $t.settings.modelSizeSmall
      : $t.settings.modelSizeLarger;
  }

  function sourceOf(option: LadderModel): string {
    if (option.source === 'bundled') return $t.settings.bundledRuntime;
    if (option.source === 'provider') return $t.settings.viaOllama;
    return $t.settings.modelStatus.plannedCandidate;
  }

  function statusOf(option: LadderModel): string {
    if (option.installed) return $t.settings.modelStatus.installed;
    if (option.source === 'planned') return $t.settings.modelStatus.plannedCandidate;
    return $t.settings.modelStatus.notInstalled;
  }

  function selected(option: LadderModel): boolean {
    if (option.bundled?.installed) {
      return status.active !== false && status.selectedModel === option.bundled.modelId;
    }
    if (option.provider) {
      return (
        providerStatus.active !== false && providerStatus.selectedModel === option.provider.name
      );
    }
    return false;
  }
</script>

<div class="model-class">
  <h3>{$t.settings.modelClass(group.minimumMemoryGb)}</h3>
  <div class="model-catalog">
    {#each group.models as option (option.entry.id)}
      {@const bundled = option.bundled}
      {@const provider = option.provider}
      {@const progress = bundled ? downloading[bundled.modelId] : undefined}
      {@const licenceSettled =
        !bundled || !bundled.acceptanceRequired || bundled.acceptedAtMs !== null}
      <article class="model-card" class:active={selected(option)}>
        <div class="model-card-copy">
          <div class="model-card-heading">
            <h3>{option.entry.name}</h3>
            {#if option.recommended}<span class="model-badge"
                >{$t.settings.recommendedForMachine}</span
              >{/if}
            {#if option.source !== 'planned'}<span class="model-badge quiet"
                >{sourceOf(option)}</span
              >{/if}
            {#if option.entry.origin === 'european'}<span class="model-badge quiet"
                >{$t.settings.european}</span
              >{/if}
          </div>
          <p>{$t.settings.modelDescription[option.entry.id]}</p>
          <div class="model-meta">
            <span>{option.entry.family}</span>
            <span>{sizeOf(option)}</span>
            <span>{$t.settings.memoryMinimum(option.minimumMemoryGb)}</span>
            <span>{$t.settings.modelLicence[option.entry.licence]}</span>
            <span>{$t.settings.modelOrigin[option.entry.origin]}</span>
            <span
              >{option.entry.languages
                .slice(0, 3)
                .map((code) => $t.settings.modelLanguage[code])
                .join(' · ')}</span
            >
          </div>
          {#if option.entry.testOnly}<p class="setting-hint">{$t.settings.testOnlyHint}</p>{/if}
          {#if !option.fits}<p class="setting-hint">
              {$t.settings.memoryTooSmall(option.minimumMemoryGb)}
            </p>{/if}
          <p class="model-evaluation">
            {option.entry.testedLanguages.length
              ? $t.settings.evaluatedIn(
                  listOf(
                    option.entry.testedLanguages.map((code) => $t.settings.modelLanguage[code]),
                  ),
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
              <a
                class="licence-link"
                href={bundled.licenceUrl}
                target="_blank"
                rel="noreferrer noopener">{$t.settings.licenceRead}</a
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
    {/each}
  </div>
</div>

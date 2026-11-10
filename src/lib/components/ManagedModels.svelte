<script lang="ts">
  import type { ModelTerms, ProtocolProviderStatus } from '../workflow/types';
  import { managedModelOptions } from '../models/managedModels';
  import { formatModelSize } from '../models/modelSize';
  import { t } from '../i18n';
  import { errorMessage } from '../errors';

  export let terms: ModelTerms[];
  export let status: ProtocolProviderStatus;
  export let memoryGb: number | null;
  export let downloading: Record<string, number>;
  export let downloadErrors: Record<string, string> = {};
  export let onAcceptLicence: (model: string) => Promise<void>;
  export let onDownloadModel: (model: string) => Promise<void>;
  export let onCancelDownload: (model: string) => Promise<void>;
  export let onChoose: (model: string) => Promise<void>;
  $: options = managedModelOptions(terms, memoryGb);
</script>

<section aria-label={$t.settings.curatedModels}>
  <p class="model-kicker">{$t.settings.bundledRuntime}</p>
  <p class="setting-hint">{$t.settings.downloadIntro}</p>
  <p class="setting-hint">
    {memoryGb === null ? $t.settings.memoryUnknown : $t.settings.memoryReported(memoryGb)}
  </p>
  {#if memoryGb !== null && memoryGb <= 8}<p class="setting-hint">{$t.settings.memoryTight}</p>{/if}
  <p class="setting-hint">{$t.settings.qualityCaution}</p>
  {#if !status.serverReachable}<p class="setting-hint">{errorMessage(status.message)}</p>{/if}
  <div class="model-catalog">
    {#each options as option (option.model.modelId)}
      {@const model = option.model}
      {@const selected =
        status.active !== false && status.selectedModel === model.modelId && model.installed}
      {@const progress = downloading[model.modelId]}
      {@const licenceSettled = !model.acceptanceRequired || model.acceptedAtMs !== null}
      <article class="model-card" class:active={selected}>
        <div class="model-card-copy">
          <div class="model-card-heading">
            <h3>{option.entry.name}</h3>
            {#if option.recommended}<span class="model-badge"
                >{$t.settings.recommendedForMachine}</span
              >{/if}
          </div>
          <p>{$t.settings.modelDescription[option.entry.id]}</p>
          <div class="model-meta">
            <span>{formatModelSize(model.byteCount)}</span>
            <span>{$t.settings.memoryMinimum(option.minimumMemoryGb)}</span>
            <span>{$t.settings.modelLicence[option.entry.licence]}</span>
          </div>
          {#if !option.fits}<p class="setting-hint">
              {$t.settings.memoryTooSmall(option.minimumMemoryGb)}
            </p>{/if}
          {#if downloadErrors[model.modelId]}
            <p class="setting-error" role="alert">{downloadErrors[model.modelId]}</p>
          {/if}
        </div>
        <div class="model-card-action">
          {#if progress !== undefined}
            <progress
              max="100"
              value={progress}
              aria-label={$t.settings.downloadModel(formatModelSize(model.byteCount))}
            ></progress>
            <span>{progress}%</span>
            <button class="quiet-action" onclick={() => onCancelDownload(model.modelId)}
              >{$t.settings.cancel}</button
            >
          {:else if model.installed}
            <span class="model-status">{$t.settings.modelStatus.installed}</span>
            <button
              class="secondary-action"
              disabled={selected || !option.fits || !status.serverReachable}
              onclick={() => onChoose(model.modelId)}
              >{selected ? $t.settings.modelSelected : $t.settings.useThisModel}</button
            >
          {:else if !licenceSettled && option.fits}
            <p>{$t.settings.licenceBody($t.settings.modelLicence[option.entry.licence])}</p>
            <a
              class="licence-link"
              href={model.licenceUrl}
              target="_blank"
              rel="noreferrer noopener">{$t.settings.licenceRead}</a
            >
            <button class="secondary-action" onclick={() => onAcceptLicence(model.modelId)}
              >{$t.settings.licenceAccept}</button
            >
          {:else}
            <button
              class="secondary-action"
              disabled={!option.fits}
              onclick={() => onDownloadModel(model.modelId)}
              >{$t.settings.downloadModel(formatModelSize(model.byteCount))}</button
            >
          {/if}
        </div>
      </article>
    {/each}
  </div>
</section>

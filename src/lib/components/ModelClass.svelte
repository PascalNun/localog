<script lang="ts">
  import type { MemoryClass } from '../models/modelLadder';
  import type { ProtocolProviderStatus } from '../workflow/types';
  import { t } from '../i18n';
  import ModelCard from './ModelCard.svelte';

  export let group: MemoryClass;
  export let status: ProtocolProviderStatus;
  export let providerStatus: ProtocolProviderStatus;
  export let downloading: Record<string, number>;
  export let onAcceptLicence: (model: string) => Promise<void>;
  export let onDownloadModel: (model: string) => Promise<void>;
  export let onCancelDownload: (model: string) => Promise<void>;
  export let onChoose: (model: string) => Promise<void>;
  export let onChooseProvider: (model: string) => Promise<void>;
</script>

<div class="model-class">
  <h3>{$t.settings.modelClass(group.minimumMemoryGb)}</h3>
  <div class="model-catalog">
    {#each group.models as option (option.entry.id)}
      <ModelCard
        {option}
        {status}
        {providerStatus}
        {downloading}
        {onAcceptLicence}
        {onDownloadModel}
        {onCancelDownload}
        {onChoose}
        {onChooseProvider}
      />
    {/each}
  </div>
</div>

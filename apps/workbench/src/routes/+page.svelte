<script lang="ts">
  const features = [
    'Triangle review',
    'Development factor selection',
    'Candidate method comparison',
    'Accident-year selections and blends',
    'Audit trail and exports'
  ];
</script>

<svelte:head>
  <title>Rustuary Workbench</title>
</svelte:head>

<main>
  <h1>Rustuary reserving workbench</h1>
  <p>Starter UI for actuarial review workflows.</p>
  <ul>
    {#each features as feature}
      <li>{feature}</li>
    {/each}
  </ul>
</main>

<script lang="ts">
  import type { ColDef } from "ag-grid-community";
  import AgGrid from "./AgGrid.svelte";

  type OutputKind = "incremental" | "cumulative";
  type PreviewRow = Record<string, number | string | null>;

  const sourceColumns = [
    "reserving_class",
    "country",
    "coverage",
    "accident_date",
    "payment_date",
    "paid_loss",
    "currency",
  ];

  const folderPaths = ["Motor / CH / MTPL", "Property / CH / Buildings"];
  const bucketMonthOptions = ["1", "3", "6", "12"];
  const outputKindOptions: OutputKind[] = ["incremental", "cumulative"];

  const mappings = [
    ["Origin date", "accident_date"],
    ["Development date", "payment_date"],
    ["Amount", "paid_loss"],
    ["Portfolio ID", "reserving_class"],
    ["Measure", "paid"],
    ["Currency", "currency"],
  ];

  const incrementalRows: PreviewRow[] = [
    {
      portfolio: "Motor",
      segment_path: "CH / MTPL",
      origin: 2024,
      dev_12: 100,
      dev_24: 50,
    },
    {
      portfolio: "Motor",
      segment_path: "CH / MTPL",
      origin: 2025,
      dev_12: 80,
      dev_24: null,
    },
    {
      portfolio: "Property",
      segment_path: "CH / Buildings",
      origin: 2024,
      dev_12: 200,
      dev_24: 25,
    },
  ];

  const cumulativeRows: PreviewRow[] = [
    {
      portfolio: "Motor",
      segment_path: "CH / MTPL",
      origin: 2024,
      dev_12: 100,
      dev_24: 150,
    },
    {
      portfolio: "Motor",
      segment_path: "CH / MTPL",
      origin: 2025,
      dev_12: 80,
      dev_24: null,
    },
    {
      portfolio: "Property",
      segment_path: "CH / Buildings",
      origin: 2024,
      dev_12: 200,
      dev_24: 225,
    },
  ];

  const triangleColumns: ColDef<PreviewRow>[] = [
    { field: "portfolio", headerName: "Portfolio", pinned: "left" },
    {
      field: "segment_path",
      headerName: "Segments",
      pinned: "left",
      minWidth: 160,
    },
    { field: "origin", headerName: "Origin", width: 112 },
    { field: "dev_12", headerName: "12", type: "rightAligned" },
    { field: "dev_24", headerName: "24", type: "rightAligned" },
  ];

  const chainLadderRows: PreviewRow[] = [
    {
      origin: 2024,
      latest_development: 24,
      selected_factor: 1,
      latest_observed: 150,
      ultimate: 150,
      reserve: 0,
    },
    {
      origin: 2025,
      latest_development: 12,
      selected_factor: 1.875,
      latest_observed: 80,
      ultimate: 150,
      reserve: 70,
    },
  ];

  const chainLadderColumns: ColDef<PreviewRow>[] = [
    { field: "origin", headerName: "Origin", pinned: "left" },
    {
      field: "latest_development",
      headerName: "Latest age",
      type: "rightAligned",
    },
    {
      field: "selected_factor",
      headerName: "Selected factor",
      type: "rightAligned",
    },
    { field: "latest_observed", headerName: "Latest", type: "rightAligned" },
    { field: "ultimate", headerName: "Ultimate", type: "rightAligned" },
    { field: "reserve", headerName: "Reserve", type: "rightAligned" },
  ];

  let fileName = $state("raw_claim_events.csv");
  let bucketMonths = $state("12");
  let outputKind = $state<OutputKind>("cumulative");
  const triangleRows = $derived(
    outputKind === "cumulative" ? cumulativeRows : incrementalRows,
  );

  function handleFileSelection(event: Event) {
    const input = event.currentTarget as HTMLInputElement;
    fileName = input.files?.[0]?.name ?? "No CSV selected";
  }
</script>

<svelte:head>
  <title>Rustuary playground</title>
</svelte:head>

<main class="min-h-screen bg-slate-100 text-slate-950">
  <section
    class="border-b border-amber-200 bg-amber-50 px-4 py-3 text-sm text-amber-950"
  >
    <div class="mx-auto flex max-w-7xl flex-wrap items-center gap-3">
      <strong class="rounded-md bg-amber-200 px-2 py-1 text-xs uppercase"
        >Playground</strong
      >
      <span
        >Non-production preview. Results are sample outputs and are not
        audit-controlled.</span
      >
    </div>
  </section>

  <div
    class="mx-auto grid max-w-7xl gap-4 px-4 py-5 lg:grid-cols-[320px_minmax(0,1fr)]"
  >
    <aside class="space-y-4">
      <section
        class="rounded-lg border border-slate-200 bg-white p-4 shadow-sm"
      >
        <h1 class="text-lg font-semibold text-slate-950">
          Rustuary playground
        </h1>
        <p class="mt-1 text-sm text-slate-600">
          Triangle build and chain ladder review shell.
        </p>
      </section>

      <section
        class="rounded-lg border border-slate-200 bg-white p-4 shadow-sm"
      >
        <h2 class="text-sm font-semibold text-slate-800">CSV upload</h2>
        <label class="mt-3 block">
          <span class="text-xs font-medium text-slate-600">Claims CSV</span>
          <input
            class="mt-1 block w-full rounded-md border border-slate-300 bg-white px-3 py-2 text-sm"
            type="file"
            accept=".csv,text/csv"
            onchange={handleFileSelection}
          />
        </label>
        <p class="mt-2 truncate text-xs text-slate-500">{fileName}</p>
      </section>

      <section
        class="rounded-lg border border-slate-200 bg-white p-4 shadow-sm"
      >
        <h2 class="text-sm font-semibold text-slate-800">Column mapping</h2>
        <div class="mt-3 space-y-3">
          {#each mappings as [label, selected]}
            <label class="block">
              <span class="text-xs font-medium text-slate-600">{label}</span>
              <select
                class="mt-1 w-full rounded-md border border-slate-300 bg-white px-3 py-2 text-sm"
                value={selected}
              >
                {#if selected === "paid"}
                  <option value="paid">paid</option>
                {/if}
                {#each sourceColumns as column}
                  <option value={column}>{column}</option>
                {/each}
              </select>
            </label>
          {/each}
        </div>
      </section>

      <section
        class="rounded-lg border border-slate-200 bg-white p-4 shadow-sm"
      >
        <h2 class="text-sm font-semibold text-slate-800">Build settings</h2>
        <div class="mt-3">
          <label class="text-xs font-medium text-slate-600" for="portfolio-id"
            >Portfolio ID</label
          >
          <select
            id="portfolio-id"
            class="mt-1 w-full rounded-md border border-slate-300 bg-white px-3 py-2 text-sm"
            value="reserving_class"
          >
            {#each sourceColumns as column}
              <option value={column}>{column}</option>
            {/each}
          </select>
        </div>

        <fieldset class="mt-4">
          <legend class="text-xs font-medium text-slate-600"
            >Ordered segments</legend
          >
          <div class="mt-2 grid grid-cols-2 gap-2">
            <label
              class="flex items-center gap-2 rounded-md border border-slate-200 px-3 py-2 text-sm"
            >
              <input checked type="checkbox" />
              country
            </label>
            <label
              class="flex items-center gap-2 rounded-md border border-slate-200 px-3 py-2 text-sm"
            >
              <input checked type="checkbox" />
              coverage
            </label>
          </div>
        </fieldset>

        <fieldset class="mt-4">
          <legend class="text-xs font-medium text-slate-600">Bucket size</legend
          >
          <div class="mt-2 grid grid-cols-4 gap-1 rounded-lg bg-slate-100 p-1">
            {#each bucketMonthOptions as value}
              <label
                class="cursor-pointer rounded-md px-2 py-2 text-center text-sm font-medium {bucketMonths ===
                value
                  ? 'bg-white text-slate-950 shadow-sm'
                  : 'text-slate-600'}"
              >
                <input
                  class="sr-only"
                  type="radio"
                  bind:group={bucketMonths}
                  {value}
                />
                {value}
              </label>
            {/each}
          </div>
        </fieldset>

        <fieldset class="mt-4">
          <legend class="text-xs font-medium text-slate-600"
            >Output basis</legend
          >
          <div class="mt-2 grid grid-cols-2 gap-1 rounded-lg bg-slate-100 p-1">
            {#each outputKindOptions as value}
              <label
                class="cursor-pointer rounded-md px-2 py-2 text-center text-sm font-medium {outputKind ===
                value
                  ? 'bg-white text-slate-950 shadow-sm'
                  : 'text-slate-600'}"
              >
                <input
                  class="sr-only"
                  type="radio"
                  bind:group={outputKind}
                  {value}
                />
                {value}
              </label>
            {/each}
          </div>
        </fieldset>
      </section>
    </aside>

    <section class="space-y-4">
      <div class="grid gap-4 xl:grid-cols-[minmax(0,1fr)_280px]">
        <section
          class="rounded-lg border border-slate-200 bg-white p-4 shadow-sm"
        >
          <div class="mb-3 flex flex-wrap items-center justify-between gap-2">
            <h2 class="text-sm font-semibold text-slate-800">
              Triangle preview
            </h2>
            <span
              class="rounded-md bg-slate-100 px-2 py-1 text-xs text-slate-600"
            >
              {bucketMonths} month / {outputKind}
            </span>
          </div>
          <AgGrid
            label="Triangle preview"
            rows={triangleRows}
            columns={triangleColumns}
          />
        </section>

        <section
          class="rounded-lg border border-slate-200 bg-white p-4 shadow-sm"
        >
          <h2 class="text-sm font-semibold text-slate-800">Folder preview</h2>
          <ol class="mt-3 space-y-2">
            {#each folderPaths as path}
              <li
                class="rounded-md border border-slate-200 bg-slate-50 px-3 py-2 text-sm text-slate-700"
              >
                {path}
              </li>
            {/each}
          </ol>
        </section>
      </div>

      <section
        class="rounded-lg border border-slate-200 bg-white p-4 shadow-sm"
      >
        <div class="mb-3 flex flex-wrap items-center justify-between gap-2">
          <h2 class="text-sm font-semibold text-slate-800">
            Chain ladder result preview
          </h2>
          <span
            class="rounded-md bg-slate-100 px-2 py-1 text-xs text-slate-600"
          >
            Sample factors
          </span>
        </div>
        <AgGrid
          label="Chain ladder result preview"
          rows={chainLadderRows}
          columns={chainLadderColumns}
          height="280px"
        />
      </section>
    </section>
  </div>
</main>

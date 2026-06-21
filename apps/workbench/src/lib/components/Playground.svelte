<script lang="ts">
  import type { ColDef, ValueFormatterParams } from "ag-grid-community";
  import AgGrid from "./AgGrid.svelte";

  type OutputKind = "incremental" | "cumulative";
  type SampleDepth = "review" | "stress";
  type PreviewRow = Record<string, number | string | null>;
  type MappingKey =
    | "origin_date"
    | "development_date"
    | "amount"
    | "portfolio_id"
    | "measure"
    | "currency";
  type MappingConfig = Record<MappingKey, string>;

  const REVIEW_DEVELOPMENT_MONTHS = 36;
  const STRESS_DEVELOPMENT_MONTHS = 500;
  const REVIEW_ORIGIN_COUNT = 8;
  const STRESS_ORIGIN_COUNT = 36;

  const sourceColumns = [
    "reserving_class",
    "country",
    "coverage",
    "accident_date",
    "payment_date",
    "paid_loss",
    "currency",
  ];

  const mappingFields: { key: MappingKey; label: string }[] = [
    { key: "origin_date", label: "Origin date" },
    { key: "development_date", label: "Development date" },
    { key: "amount", label: "Amount" },
    { key: "portfolio_id", label: "Portfolio ID" },
    { key: "measure", label: "Measure" },
    { key: "currency", label: "Currency" },
  ];

  const bucketMonthOptions = ["1", "3", "6", "12"];
  const outputKindOptions: OutputKind[] = ["incremental", "cumulative"];
  const sampleDepthOptions: { value: SampleDepth; label: string }[] = [
    { value: "stress", label: "500-month stress" },
    { value: "review", label: "Compact review" },
  ];

  const chainLadderRows: PreviewRow[] = [
    {
      origin: "2020-01",
      latest_development: 500,
      selected_factor: 1,
      latest_observed: 7120,
      ultimate: 7120,
      reserve: 0,
    },
    {
      origin: "2022-01",
      latest_development: 333,
      selected_factor: 1.08,
      latest_observed: 6240,
      ultimate: 6739,
      reserve: 499,
    },
    {
      origin: "2024-01",
      latest_development: 167,
      selected_factor: 1.21,
      latest_observed: 4380,
      ultimate: 5300,
      reserve: 920,
    },
  ];

  const chainLadderColumns: ColDef<PreviewRow>[] = [
    { field: "origin", headerName: "Origin", pinned: "left", width: 120 },
    {
      field: "latest_development",
      headerName: "Latest month",
      type: "rightAligned",
      width: 132,
    },
    {
      field: "selected_factor",
      headerName: "Selected factor",
      type: "rightAligned",
      width: 144,
    },
    { field: "latest_observed", headerName: "Latest", type: "rightAligned" },
    { field: "ultimate", headerName: "Ultimate", type: "rightAligned" },
    { field: "reserve", headerName: "Reserve", type: "rightAligned" },
  ];

  let fileName = $state("synthetic_500_month_claims.csv");
  let bucketMonths = $state("1");
  let outputKind = $state<OutputKind>("cumulative");
  let sampleDepth = $state<SampleDepth>("stress");
  let selectedMappings = $state<MappingConfig>({
    origin_date: "accident_date",
    development_date: "payment_date",
    amount: "paid_loss",
    portfolio_id: "reserving_class",
    measure: "paid",
    currency: "currency",
  });
  let selectedSegments = $state<string[]>(["country", "coverage"]);

  const sourceDevelopmentMonths = $derived(
    sampleDepth === "stress"
      ? STRESS_DEVELOPMENT_MONTHS
      : REVIEW_DEVELOPMENT_MONTHS,
  );
  const originCount = $derived(
    sampleDepth === "stress" ? STRESS_ORIGIN_COUNT : REVIEW_ORIGIN_COUNT,
  );
  const developmentAges = $derived(
    createDevelopmentAges(sourceDevelopmentMonths, Number(bucketMonths)),
  );
  const triangleColumns = $derived(
    createTriangleColumns(developmentAges, selectedSegments),
  );
  const triangleRows = $derived(
    createTriangleRows({
      basis: outputKind,
      developmentAges,
      mappings: selectedMappings,
      originCount,
      segments: selectedSegments,
    }),
  );
  const displayedCellCount = $derived(originCount * developmentAges.length);
  const folderPaths = $derived(
    createFolderPaths(triangleRows, selectedSegments),
  );
  const latestDevelopmentMonth = $derived(developmentAges.at(-1) ?? 0);
  const triangleGridKey = $derived(
    [
      bucketMonths,
      outputKind,
      sampleDepth,
      selectedSegments.join(","),
      ...mappingFields.map((field) => selectedMappings[field.key]),
    ].join("|"),
  );

  function handleFileSelection(event: Event) {
    const input = event.currentTarget as HTMLInputElement;
    fileName = input.files?.[0]?.name ?? "No CSV selected";
  }

  function selectBucketMonths(value: string) {
    bucketMonths = value;
  }

  function selectOutputKind(value: OutputKind) {
    outputKind = value;
  }

  function updateMapping(key: MappingKey, value: string) {
    selectedMappings = { ...selectedMappings, [key]: value };
  }

  function toggleSegment(segment: string) {
    selectedSegments = selectedSegments.includes(segment)
      ? selectedSegments.filter((item) => item !== segment)
      : [...selectedSegments, segment];
  }

  function ageField(age: number) {
    return `dev_${age.toString().padStart(3, "0")}`;
  }

  function createDevelopmentAges(sourceMonths: number, bucketSize: number) {
    const columnCount = Math.ceil(sourceMonths / bucketSize);
    return Array.from(
      { length: columnCount },
      (_, index) => (index + 1) * bucketSize,
    );
  }

  function createTriangleColumns(
    developmentAges: number[],
    segments: string[],
  ): ColDef<PreviewRow>[] {
    const identityColumns: ColDef<PreviewRow>[] = [
      {
        field: "portfolio_id",
        headerName: "Portfolio",
        pinned: "left",
        width: 136,
      },
      {
        field: "origin_month",
        headerName: "Origin",
        pinned: "left",
        width: 116,
      },
      {
        field: "latest_month",
        headerName: "Latest",
        pinned: "left",
        type: "rightAligned",
        width: 96,
      },
    ];

    for (const segment of segments) {
      identityColumns.splice(identityColumns.length - 2, 0, {
        field: segment,
        headerName: segmentTitle(segment),
        pinned: "left",
        width: 128,
      });
    }

    return [
      ...identityColumns,
      ...developmentAges.map((age) => ({
        field: ageField(age),
        headerName: `M${age.toString().padStart(3, "0")}`,
        minWidth: 78,
        width: 84,
        type: "rightAligned",
        valueFormatter: ({
          value,
        }: ValueFormatterParams<PreviewRow, number | null>) =>
          formatAmount(value),
      })),
    ];
  }

  function createTriangleRows({
    basis,
    developmentAges,
    mappings,
    originCount,
    segments,
  }: {
    basis: OutputKind;
    developmentAges: number[];
    mappings: MappingConfig;
    originCount: number;
    segments: string[];
  }): PreviewRow[] {
    return Array.from({ length: originCount }, (_, originIndex) => {
      const originDate = new Date(2020, originIndex, 1);
      const sourceRecord = createSourceRecord(originIndex, originDate);
      const observedCount = Math.max(
        3,
        developmentAges.length -
          Math.floor(
            originIndex *
              (developmentAges.length / Math.max(1, originCount - 1)),
          ),
      );
      let cumulative = 0;
      const row: PreviewRow = {
        portfolio_id: resolveMappingValue(sourceRecord, mappings.portfolio_id),
        origin_month: formatOriginMonth(originDate),
        latest_month:
          developmentAges[
            Math.min(observedCount - 1, developmentAges.length - 1)
          ],
      };

      for (const segment of segments) {
        row[segment] = resolveMappingValue(sourceRecord, segment);
      }

      for (const [ageIndex, age] of developmentAges.entries()) {
        if (ageIndex >= observedCount) {
          row[ageField(age)] = null;
          continue;
        }

        const incremental = incrementalAmount(originIndex, ageIndex);
        cumulative += incremental;
        row[ageField(age)] =
          basis === "cumulative" ? Math.round(cumulative) : incremental;
      }

      return row;
    });
  }

  function createFolderPaths(rows: PreviewRow[], segments: string[]) {
    return Array.from(
      new Set(
        rows.map((row) =>
          [row.portfolio_id, ...segments.map((segment) => row[segment])]
            .filter((value) => value !== null && value !== "")
            .join(" / "),
        ),
      ),
    );
  }

  function createSourceRecord(originIndex: number, originDate: Date) {
    return {
      reserving_class: originIndex % 5 === 0 ? "Property" : "Motor",
      country: originIndex % 4 === 0 ? "DE" : "CH",
      coverage: originIndex % 3 === 0 ? "Casco" : "MTPL",
      accident_date: formatOriginMonth(originDate),
      payment_date: formatOriginMonth(
        new Date(originDate.getFullYear(), originDate.getMonth() + 1),
      ),
      paid_loss: (125 + originIndex * 4).toString(),
      currency: originIndex % 4 === 0 ? "EUR" : "CHF",
      paid: "paid",
    };
  }

  function resolveMappingValue(
    sourceRecord: Record<string, string>,
    sourceColumn: string,
  ) {
    return sourceRecord[sourceColumn] ?? sourceColumn;
  }

  function segmentTitle(segment: string) {
    return segment
      .split("_")
      .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
      .join(" ");
  }

  function incrementalAmount(originIndex: number, ageIndex: number) {
    const originScale = 125 + originIndex * 4;
    const decay = Math.exp(-ageIndex / 92);
    const seasonality = 10 * Math.sin((ageIndex + originIndex) / 6);
    return Math.max(1, Math.round(originScale * decay + seasonality));
  }

  function formatOriginMonth(date: Date) {
    return `${date.getFullYear()}-${(date.getMonth() + 1).toString().padStart(2, "0")}`;
  }

  function formatAmount(value: number | null | undefined) {
    if (typeof value !== "number") {
      return "";
    }

    return value.toLocaleString("en-US", { maximumFractionDigits: 0 });
  }
</script>

<svelte:head>
  <title>Rustuary playground</title>
</svelte:head>

<main class="min-h-screen bg-slate-100 text-slate-950">
  <section
    class="border-b border-amber-200 bg-amber-50 px-4 py-3 text-sm text-amber-950"
  >
    <div class="mx-auto flex max-w-[96rem] flex-wrap items-center gap-3">
      <strong class="rounded-md bg-amber-200 px-2 py-1 text-xs uppercase"
        >Playground</strong
      >
      <span
        >Non-production preview. Results are sample outputs and are not
        audit-controlled.</span
      >
    </div>
  </section>

  <div class="mx-auto max-w-[96rem] space-y-4 px-4 py-5">
    <header class="flex flex-wrap items-end justify-between gap-4">
      <div>
        <h1 class="text-lg font-semibold text-slate-950">
          Rustuary playground
        </h1>
        <p class="mt-1 text-sm text-slate-600">
          Triangle build and chain ladder review shell.
        </p>
      </div>
      <dl class="grid grid-cols-2 gap-2 text-sm md:grid-cols-4">
        <div
          class="rounded-lg border border-slate-200 bg-white px-3 py-2 shadow-sm"
        >
          <dt class="text-xs text-slate-500">Development</dt>
          <dd class="font-semibold text-slate-950">
            {latestDevelopmentMonth} months
          </dd>
        </div>
        <div
          class="rounded-lg border border-slate-200 bg-white px-3 py-2 shadow-sm"
        >
          <dt class="text-xs text-slate-500">Origins</dt>
          <dd class="font-semibold text-slate-950">{originCount}</dd>
        </div>
        <div
          class="rounded-lg border border-slate-200 bg-white px-3 py-2 shadow-sm"
        >
          <dt class="text-xs text-slate-500">Grid cells</dt>
          <dd class="font-semibold text-slate-950">
            {displayedCellCount.toLocaleString("en-US")}
          </dd>
        </div>
        <div
          class="rounded-lg border border-slate-200 bg-white px-3 py-2 shadow-sm"
        >
          <dt class="text-xs text-slate-500">Basis</dt>
          <dd class="font-semibold text-slate-950">{outputKind}</dd>
        </div>
      </dl>
    </header>

    <section
      class="grid gap-3 xl:grid-cols-[minmax(0,1.4fr)_minmax(0,1fr)_320px]"
    >
      <div class="rounded-lg border border-slate-200 bg-white p-4 shadow-sm">
        <h2 class="text-sm font-semibold text-slate-800">CSV and sample</h2>
        <div class="mt-3 grid gap-3 md:grid-cols-2">
          <label class="block">
            <span class="text-xs font-medium text-slate-600">Claims CSV</span>
            <input
              class="mt-1 block w-full rounded-md border border-slate-300 bg-white px-3 py-2 text-sm"
              type="file"
              accept=".csv,text/csv"
              onchange={handleFileSelection}
            />
          </label>
          <label class="block">
            <span class="text-xs font-medium text-slate-600">Sample depth</span>
            <select
              class="mt-1 w-full rounded-md border border-slate-300 bg-white px-3 py-2 text-sm"
              bind:value={sampleDepth}
            >
              {#each sampleDepthOptions as option}
                <option value={option.value}>{option.label}</option>
              {/each}
            </select>
          </label>
        </div>
        <p class="mt-2 truncate text-xs text-slate-500">{fileName}</p>
      </div>

      <div class="rounded-lg border border-slate-200 bg-white p-4 shadow-sm">
        <h2 class="text-sm font-semibold text-slate-800">Column mapping</h2>
        <div class="mt-3 grid gap-2 md:grid-cols-2">
          {#each mappingFields as field}
            <label class="block">
              <span class="text-xs font-medium text-slate-600"
                >{field.label}</span
              >
              <select
                class="mt-1 w-full rounded-md border border-slate-300 bg-white px-3 py-2 text-sm"
                value={selectedMappings[field.key]}
                onchange={(event) =>
                  updateMapping(field.key, event.currentTarget.value)}
              >
                {#if field.key === "measure"}
                  <option value="paid">paid</option>
                {/if}
                {#each sourceColumns as column}
                  <option value={column}>{column}</option>
                {/each}
              </select>
            </label>
          {/each}
        </div>
      </div>

      <div class="rounded-lg border border-slate-200 bg-white p-4 shadow-sm">
        <h2 class="text-sm font-semibold text-slate-800">Build settings</h2>
        <fieldset class="mt-3">
          <legend class="text-xs font-medium text-slate-600">
            Ordered segments
          </legend>
          <div class="mt-2 grid grid-cols-2 gap-2">
            <label
              class="flex items-center gap-2 rounded-md border border-slate-200 px-3 py-2 text-sm"
            >
              <input
                checked={selectedSegments.includes("country")}
                type="checkbox"
                onchange={() => toggleSegment("country")}
              />
              country
            </label>
            <label
              class="flex items-center gap-2 rounded-md border border-slate-200 px-3 py-2 text-sm"
            >
              <input
                checked={selectedSegments.includes("coverage")}
                type="checkbox"
                onchange={() => toggleSegment("coverage")}
              />
              coverage
            </label>
          </div>
        </fieldset>
        <fieldset class="mt-4">
          <legend class="text-xs font-medium text-slate-600">Bucket size</legend
          >
          <div class="mt-2 grid grid-cols-4 gap-1 rounded-lg bg-slate-100 p-1">
            {#each bucketMonthOptions as value}
              <button
                type="button"
                onclick={() => selectBucketMonths(value)}
                class="cursor-pointer rounded-md px-2 py-2 text-center text-sm font-medium {bucketMonths ===
                value
                  ? 'bg-white text-slate-950 shadow-sm'
                  : 'text-slate-600'}"
              >
                {value}
              </button>
            {/each}
          </div>
        </fieldset>
        <fieldset class="mt-4">
          <legend class="text-xs font-medium text-slate-600">
            Output basis
          </legend>
          <div class="mt-2 grid grid-cols-2 gap-1 rounded-lg bg-slate-100 p-1">
            {#each outputKindOptions as value}
              <button
                type="button"
                onclick={() => selectOutputKind(value)}
                class="cursor-pointer rounded-md px-2 py-2 text-center text-sm font-medium {outputKind ===
                value
                  ? 'bg-white text-slate-950 shadow-sm'
                  : 'text-slate-600'}"
              >
                {value}
              </button>
            {/each}
          </div>
        </fieldset>
      </div>
    </section>

    <section class="rounded-lg border border-slate-200 bg-white p-4 shadow-sm">
      <div class="mb-3 flex flex-wrap items-center justify-between gap-2">
        <div>
          <h2 class="text-sm font-semibold text-slate-800">Triangle preview</h2>
          <p class="mt-1 text-xs text-slate-500">
            {triangleRows.length} origin rows x {developmentAges.length} development
            columns
          </p>
        </div>
        <span class="rounded-md bg-slate-100 px-2 py-1 text-xs text-slate-600">
          {bucketMonths} month / {outputKind}
        </span>
      </div>
      {#key triangleGridKey}
        <AgGrid
          label="Triangle preview"
          rows={triangleRows}
          columns={triangleColumns}
          height="calc(100vh - 260px)"
        />
      {/key}
    </section>

    <section class="grid gap-4 xl:grid-cols-[320px_minmax(0,1fr)]">
      <div class="rounded-lg border border-slate-200 bg-white p-4 shadow-sm">
        <h2 class="text-sm font-semibold text-slate-800">Folder preview</h2>
        <ol class="mt-3 max-h-64 space-y-2 overflow-auto">
          {#each folderPaths as path}
            <li
              class="rounded-md border border-slate-200 bg-slate-50 px-3 py-2 text-sm text-slate-700"
            >
              {path}
            </li>
          {/each}
        </ol>
      </div>

      <div class="rounded-lg border border-slate-200 bg-white p-4 shadow-sm">
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
          height="300px"
        />
      </div>
    </section>
  </div>
</main>

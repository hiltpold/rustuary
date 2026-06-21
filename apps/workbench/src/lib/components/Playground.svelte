<script lang="ts">
  import type { ColDef, ValueFormatterParams } from "ag-grid-community";
  import AgGrid from "./AgGrid.svelte";

  type OutputKind = "incremental" | "cumulative";
  type SampleDepth = "review" | "stress";
  type PreviewRow = Record<string, number | string | null>;
  type SourceRecord = Record<string, string>;
  type SegmentDefinition = { id: string; name: string; source: string };
  type TriangleKeyOption = {
    key: string;
    label: string;
    portfolioId: string;
    segments: { name: string; value: string }[];
    originRows: number;
  };
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
  let selectedSegments = $state<SegmentDefinition[]>([
    createSegmentDefinition("country"),
    createSegmentDefinition("coverage"),
  ]);
  let nextSegmentSource = $state("currency");
  let selectedTriangleKey = $state("");

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
  const triangleKeyOptions = $derived(
    createTriangleKeyOptions(triangleRows, selectedSegments),
  );
  const activeTriangleKey = $derived(
    triangleKeyOptions.some((option) => option.key === selectedTriangleKey)
      ? selectedTriangleKey
      : (triangleKeyOptions[0]?.key ?? ""),
  );
  const activeTriangleRows = $derived(
    triangleRows.filter(
      (row) => createTriangleKey(row, selectedSegments) === activeTriangleKey,
    ),
  );
  const displayedCellCount = $derived(
    activeTriangleRows.length * developmentAges.length,
  );
  const activeTriangleLabel = $derived(
    triangleKeyOptions.find((option) => option.key === activeTriangleKey)
      ?.label ?? "No triangle selected",
  );
  const latestDevelopmentMonth = $derived(developmentAges.at(-1) ?? 0);
  const segmentSourceOptions = $derived(
    sourceColumns.filter(
      (column) => !isReservedSegmentSource(column, selectedMappings),
    ),
  );
  const availableSegmentSources = $derived(
    segmentSourceOptions.filter(
      (column) =>
        !selectedSegments.some((segment) => segment.source === column),
    ),
  );

  $effect(() => {
    if (
      availableSegmentSources.length > 0 &&
      !availableSegmentSources.includes(nextSegmentSource)
    ) {
      nextSegmentSource = availableSegmentSources[0];
    }
  });

  $effect(() => {
    if (activeTriangleKey !== selectedTriangleKey) {
      selectedTriangleKey = activeTriangleKey;
    }
  });

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

  function addSegment() {
    const sourceColumn = availableSegmentSources.includes(nextSegmentSource)
      ? nextSegmentSource
      : availableSegmentSources[0];

    if (!sourceColumn) {
      return;
    }

    selectedSegments = [
      ...selectedSegments,
      createSegmentDefinition(sourceColumn),
    ];
  }

  function removeSegment(index: number) {
    selectedSegments = selectedSegments.filter(
      (_, itemIndex) => itemIndex !== index,
    );
  }

  function moveSegment(index: number, direction: -1 | 1) {
    const nextIndex = index + direction;
    if (nextIndex < 0 || nextIndex >= selectedSegments.length) {
      return;
    }

    const nextSegments = [...selectedSegments];
    const [segment] = nextSegments.splice(index, 1);
    nextSegments.splice(nextIndex, 0, segment);
    selectedSegments = nextSegments;
  }

  function selectTriangleKey(key: string) {
    selectedTriangleKey = key;
  }

  function updateSegmentName(index: number, name: string) {
    selectedSegments = selectedSegments.map((segment, itemIndex) =>
      itemIndex === index ? { ...segment, name } : segment,
    );
  }

  function updateSegmentSource(index: number, source: string) {
    if (!sourceOptionsForSegment(index).includes(source)) {
      return;
    }

    selectedSegments = selectedSegments.map((segment, itemIndex) => {
      if (itemIndex !== index) {
        return segment;
      }

      const previousDefaultName = segmentTitle(segment.source);
      const shouldRefreshName =
        segment.name.trim() === "" || segment.name === previousDefaultName;

      return {
        id: makeSegmentId(source),
        name: shouldRefreshName ? segmentTitle(source) : segment.name,
        source,
      };
    });
  }

  function sourceOptionsForSegment(index: number) {
    const currentSource = selectedSegments[index]?.source;
    const options = segmentSourceOptions.filter(
      (column) =>
        column === currentSource ||
        !selectedSegments.some(
          (segment, itemIndex) =>
            itemIndex !== index && segment.source === column,
        ),
    );

    if (currentSource && !options.includes(currentSource)) {
      return [currentSource, ...options];
    }

    return options;
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
    segments: SegmentDefinition[],
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
        field: segment.id,
        headerName: displaySegmentName(segment),
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
    segments: SegmentDefinition[];
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
        row[segment.id] = resolveMappingValue(sourceRecord, segment.source);
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

  function createTriangleKeyOptions(
    rows: PreviewRow[],
    segments: SegmentDefinition[],
  ): TriangleKeyOption[] {
    const options = new Map<string, TriangleKeyOption>();

    for (const row of rows) {
      const portfolioId = String(row.portfolio_id ?? "");
      const segmentValues = segments.map((segment) => ({
        name: displaySegmentName(segment),
        value: String(row[segment.id] ?? ""),
      }));
      const label = formatTrianglePathLabel(portfolioId, segmentValues);
      const key = createTriangleKeyFromParts(portfolioId, segmentValues);

      const existingOption = options.get(key);
      if (existingOption) {
        existingOption.originRows += 1;
        continue;
      }

      options.set(key, {
        key,
        label,
        portfolioId,
        segments: segmentValues,
        originRows: 1,
      });
    }

    return Array.from(options.values()).sort((left, right) =>
      left.label.localeCompare(right.label),
    );
  }

  function createTriangleKey(row: PreviewRow, segments: SegmentDefinition[]) {
    return createTriangleKeyFromParts(
      String(row.portfolio_id ?? ""),
      segments.map((segment) => ({
        name: displaySegmentName(segment),
        value: String(row[segment.id] ?? ""),
      })),
    );
  }

  function createTriangleKeyFromParts(
    portfolioId: string,
    segments: { name: string; value: string }[],
  ) {
    return JSON.stringify({
      portfolio_id: portfolioId,
      segments,
    });
  }

  function createSegmentDefinition(sourceColumn: string): SegmentDefinition {
    return {
      id: makeSegmentId(sourceColumn),
      name: segmentTitle(sourceColumn),
      source: sourceColumn,
    };
  }

  function makeSegmentId(sourceColumn: string) {
    return `segment_${sourceColumn.replace(/[^a-zA-Z0-9_]/g, "_")}`;
  }

  function isReservedSegmentSource(column: string, mappings: MappingConfig) {
    return [
      mappings.origin_date,
      mappings.development_date,
      mappings.amount,
      mappings.portfolio_id,
    ].includes(column);
  }

  function displaySegmentName(segment: SegmentDefinition) {
    return segment.name.trim() || segmentTitle(segment.source);
  }

  function formatTrianglePathLabel(
    portfolioId: string,
    segments: { name: string; value: string }[],
  ) {
    return [
      `portfolio_id=${portfolioId || "unmapped"}`,
      ...segments.map(
        (segment) => `${segment.name}=${segment.value || "blank"}`,
      ),
    ].join(" / ");
  }

  function createSourceRecord(
    originIndex: number,
    originDate: Date,
  ): SourceRecord {
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
    sourceRecord: SourceRecord,
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
      <dl class="grid grid-cols-2 gap-2 text-sm md:grid-cols-5">
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
          <dt class="text-xs text-slate-500">Paths</dt>
          <dd class="font-semibold text-slate-950">
            {triangleKeyOptions.length}
          </dd>
        </div>
        <div
          class="rounded-lg border border-slate-200 bg-white px-3 py-2 shadow-sm"
        >
          <dt class="text-xs text-slate-500">Origin rows</dt>
          <dd class="font-semibold text-slate-950">
            {activeTriangleRows.length}
          </dd>
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

    <details class="rounded-lg border border-slate-200 bg-white shadow-sm">
      <summary
        class="flex cursor-pointer flex-wrap items-center justify-between gap-3 px-4 py-3"
      >
        <div class="min-w-0">
          <h2 class="text-sm font-semibold text-slate-800">Data setup</h2>
          <p class="mt-1 truncate text-xs text-slate-500">{fileName}</p>
        </div>
        <dl class="grid grid-cols-3 gap-2 text-xs">
          <div class="rounded-md bg-slate-100 px-2 py-1">
            <dt class="text-slate-500">Sample</dt>
            <dd class="font-semibold text-slate-900">{sampleDepth}</dd>
          </div>
          <div class="rounded-md bg-slate-100 px-2 py-1">
            <dt class="text-slate-500">Fields</dt>
            <dd class="font-semibold text-slate-900">{mappingFields.length}</dd>
          </div>
          <div class="rounded-md bg-slate-100 px-2 py-1">
            <dt class="text-slate-500">Segments</dt>
            <dd class="font-semibold text-slate-900">
              {selectedSegments.length}/{segmentSourceOptions.length}
            </dd>
          </div>
        </dl>
      </summary>

      <div class="border-t border-slate-200 px-4 py-4">
        <div
          class="grid gap-5 xl:grid-cols-[minmax(0,0.8fr)_minmax(0,1.15fr)_minmax(0,1.6fr)]"
        >
          <section>
            <h3 class="text-sm font-semibold text-slate-800">CSV and sample</h3>
            <div class="mt-3 grid gap-3">
              <label class="block">
                <span class="text-xs font-medium text-slate-600">
                  Claims CSV
                </span>
                <input
                  class="mt-1 block w-full rounded-md border border-slate-300 bg-white px-3 py-2 text-sm"
                  type="file"
                  accept=".csv,text/csv"
                  onchange={handleFileSelection}
                />
              </label>
              <label class="block">
                <span class="text-xs font-medium text-slate-600">
                  Sample depth
                </span>
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
          </section>

          <section>
            <div class="flex flex-wrap items-center justify-between gap-2">
              <h3 class="text-sm font-semibold text-slate-800">
                Required mapping
              </h3>
              <span
                class="rounded-md bg-slate-100 px-2 py-1 text-xs text-slate-600"
              >
                {mappingFields.length} fields
              </span>
            </div>
            <div class="mt-3 grid gap-2 sm:grid-cols-2 xl:grid-cols-1">
              {#each mappingFields as field}
                <label class="block">
                  <span class="text-xs font-medium text-slate-600">
                    {field.label}
                  </span>
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
          </section>

          <section>
            <div class="flex flex-wrap items-center justify-between gap-2">
              <h3 class="text-sm font-semibold text-slate-800">
                Segment mapping
              </h3>
              <span
                class="rounded-md bg-slate-100 px-2 py-1 text-xs text-slate-600"
              >
                {selectedSegments.length}/{segmentSourceOptions.length}
              </span>
            </div>
            <fieldset class="mt-3">
              <legend class="text-xs font-medium text-slate-600">
                Ordered dimensions
              </legend>
              <div class="mt-2 space-y-2">
                {#each selectedSegments as segment, index}
                  <div
                    class="grid gap-2 rounded-md bg-slate-50 p-2 text-sm md:grid-cols-[2rem_minmax(7rem,1fr)_minmax(8rem,1fr)_auto]"
                  >
                    <span class="self-end text-xs font-semibold text-slate-500">
                      #{index + 1}
                    </span>
                    <label class="block">
                      <span class="text-xs font-medium text-slate-600">
                        Segment name
                      </span>
                      <input
                        class="mt-1 w-full rounded-md border border-slate-300 bg-white px-3 py-2 text-sm"
                        value={segment.name}
                        oninput={(event) =>
                          updateSegmentName(index, event.currentTarget.value)}
                      />
                    </label>
                    <label class="block">
                      <span class="text-xs font-medium text-slate-600">
                        Source column
                      </span>
                      <select
                        class="mt-1 w-full rounded-md border border-slate-300 bg-white px-3 py-2 text-sm"
                        value={segment.source}
                        onchange={(event) =>
                          updateSegmentSource(index, event.currentTarget.value)}
                      >
                        {#each sourceOptionsForSegment(index) as column}
                          <option value={column}>{column}</option>
                        {/each}
                      </select>
                    </label>
                    <span class="flex items-end gap-1">
                      <button
                        type="button"
                        class="rounded-md border border-slate-200 bg-white px-2 py-2 text-xs text-slate-600 disabled:text-slate-300"
                        disabled={index === 0}
                        onclick={() => moveSegment(index, -1)}
                      >
                        Up
                      </button>
                      <button
                        type="button"
                        class="rounded-md border border-slate-200 bg-white px-2 py-2 text-xs text-slate-600 disabled:text-slate-300"
                        disabled={index === selectedSegments.length - 1}
                        onclick={() => moveSegment(index, 1)}
                      >
                        Down
                      </button>
                      <button
                        type="button"
                        class="rounded-md border border-slate-200 bg-white px-2 py-2 text-xs text-slate-600"
                        onclick={() => removeSegment(index)}
                      >
                        Remove
                      </button>
                    </span>
                  </div>
                {/each}
              </div>
              <div class="mt-3 grid grid-cols-[minmax(0,1fr)_auto] gap-2">
                <select
                  class="rounded-md border border-slate-300 bg-white px-3 py-2 text-sm"
                  bind:value={nextSegmentSource}
                  aria-label="Segment source to add"
                  disabled={availableSegmentSources.length === 0}
                >
                  {#each availableSegmentSources as column}
                    <option value={column}>{column}</option>
                  {/each}
                </select>
                <button
                  type="button"
                  class="rounded-md border border-slate-300 bg-white px-3 py-2 text-sm font-medium text-slate-700 disabled:text-slate-300"
                  disabled={availableSegmentSources.length === 0}
                  onclick={addSegment}
                >
                  Add segment
                </button>
              </div>
            </fieldset>
          </section>
        </div>
      </div>
    </details>

    <section class="grid gap-4 xl:grid-cols-[20rem_minmax(0,1fr)]">
      <aside class="rounded-lg border border-slate-200 bg-white p-4 shadow-sm">
        <div class="flex flex-wrap items-center justify-between gap-2">
          <h2 class="text-sm font-semibold text-slate-800">Triangle filters</h2>
          <span
            class="rounded-md bg-slate-100 px-2 py-1 text-xs text-slate-600"
          >
            {triangleKeyOptions.length} paths
          </span>
        </div>
        <label class="mt-3 block">
          <span class="text-xs font-medium text-slate-600">Path</span>
          <select
            class="mt-1 w-full rounded-md border border-slate-300 bg-white px-3 py-2 text-sm"
            value={activeTriangleKey}
            onchange={(event) => selectTriangleKey(event.currentTarget.value)}
          >
            {#each triangleKeyOptions as option}
              <option value={option.key}>
                {option.label} ({option.originRows})
              </option>
            {/each}
          </select>
        </label>
        <dl class="mt-3 grid grid-cols-2 gap-2 text-xs">
          <div class="rounded-md bg-slate-100 px-2 py-2">
            <dt class="text-slate-500">Rows</dt>
            <dd class="font-semibold text-slate-900">
              {activeTriangleRows.length}
            </dd>
          </div>
          <div class="rounded-md bg-slate-100 px-2 py-2">
            <dt class="text-slate-500">Cells</dt>
            <dd class="font-semibold text-slate-900">
              {displayedCellCount.toLocaleString("en-US")}
            </dd>
          </div>
        </dl>
        <ol class="mt-3 max-h-[calc(100vh-28rem)] space-y-2 overflow-auto">
          {#each triangleKeyOptions as option}
            <li>
              <button
                type="button"
                class="w-full rounded-md border px-3 py-2 text-left text-sm {option.key ===
                activeTriangleKey
                  ? 'border-slate-400 bg-slate-100 text-slate-950'
                  : 'border-slate-200 bg-slate-50 text-slate-700'}"
                onclick={() => selectTriangleKey(option.key)}
              >
                <span class="block truncate">{option.label}</span>
                <span class="mt-1 block text-xs text-slate-500">
                  {option.originRows} origin rows
                </span>
              </button>
            </li>
          {/each}
        </ol>
      </aside>

      <section class="rounded-lg border border-slate-200 bg-white shadow-sm">
        <div
          class="flex flex-wrap items-start justify-between gap-3 border-b border-slate-200 px-4 py-3"
        >
          <div class="min-w-0">
            <h2 class="text-sm font-semibold text-slate-800">
              Claims triangle
            </h2>
            <p class="mt-1 truncate text-xs text-slate-500">
              {activeTriangleLabel}: {activeTriangleRows.length} origin rows x
              {developmentAges.length} development columns
            </p>
          </div>
          <div class="flex flex-wrap gap-3">
            <fieldset>
              <legend class="text-xs font-medium text-slate-600">
                Bucket size
              </legend>
              <div
                class="mt-1 grid grid-cols-4 gap-1 rounded-lg bg-slate-100 p-1"
              >
                {#each bucketMonthOptions as value}
                  <button
                    type="button"
                    onclick={() => selectBucketMonths(value)}
                    class="min-w-11 cursor-pointer rounded-md px-2 py-2 text-center text-sm font-medium {bucketMonths ===
                    value
                      ? 'bg-white text-slate-950 shadow-sm'
                      : 'text-slate-600'}"
                  >
                    {value}
                  </button>
                {/each}
              </div>
            </fieldset>
            <fieldset>
              <legend class="text-xs font-medium text-slate-600">
                Output basis
              </legend>
              <div
                class="mt-1 grid grid-cols-2 gap-1 rounded-lg bg-slate-100 p-1"
              >
                {#each outputKindOptions as value}
                  <button
                    type="button"
                    onclick={() => selectOutputKind(value)}
                    class="min-w-24 cursor-pointer rounded-md px-2 py-2 text-center text-sm font-medium {outputKind ===
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
        </div>
        <div class="p-3">
          <AgGrid
            label="Claims triangle"
            rows={activeTriangleRows}
            columns={triangleColumns}
            height="calc(100vh - 300px)"
          />
        </div>
      </section>
    </section>

    <section>
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

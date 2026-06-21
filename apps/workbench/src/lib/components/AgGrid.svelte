<script lang="ts" module>
  let modulesRegistered = false;
</script>

<script lang="ts">
  import { onMount } from "svelte";
  import type { ColDef, GridApi, GridOptions } from "ag-grid-community";

  type GridRow = Record<string, number | string | null>;

  interface Props {
    label: string;
    rows: GridRow[];
    columns: ColDef<GridRow>[];
    height?: string;
  }

  let { label, rows, columns, height = "320px" }: Props = $props();

  let gridElement: HTMLDivElement;
  let gridApi = $state<GridApi<GridRow> | undefined>();

  onMount(() => {
    let destroyed = false;

    async function mountGrid() {
      const { AllCommunityModule, ModuleRegistry, createGrid } =
        await import("ag-grid-community");

      if (destroyed) {
        return;
      }

      if (!modulesRegistered) {
        ModuleRegistry.registerModules([AllCommunityModule]);
        modulesRegistered = true;
      }

      const gridOptions: GridOptions<GridRow> = {
        rowData: rows,
        columnDefs: columns,
        defaultColDef: {
          filter: true,
          minWidth: 112,
          resizable: true,
          sortable: true,
        },
        animateRows: false,
        suppressDragLeaveHidesColumns: true,
      };

      gridApi = createGrid(gridElement, gridOptions);
    }

    void mountGrid();

    return () => {
      destroyed = true;
      gridApi?.destroy();
      gridApi = undefined;
    };
  });

  $effect(() => {
    if (!gridApi) {
      return;
    }

    gridApi.setGridOption("columnDefs", columns);
    gridApi.setGridOption("rowData", rows);
  });
</script>

<div
  class="ag-grid-shell overflow-hidden rounded-lg border border-slate-200"
  style:height
  aria-label={label}
>
  <div bind:this={gridElement} class="ag-theme-quartz ag-grid-surface"></div>
</div>

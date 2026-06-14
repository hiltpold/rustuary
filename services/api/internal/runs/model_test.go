package runs

import "testing"

func TestModelRunCarriesArtifactURIs(t *testing.T) {
	run := ModelRun{ID: "run_1", InputURI: "s3://bucket/input.parquet", OutputURI: "s3://bucket/output.parquet"}
	if run.InputURI == "" || run.OutputURI == "" {
		t.Fatal("expected artifact URIs")
	}
}

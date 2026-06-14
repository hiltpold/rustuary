package runs

// ModelRun is metadata for an actuarial engine run.
// Large inputs and outputs should be stored in object storage as Arrow/Parquet artifacts.
type ModelRun struct {
	ID            string
	PortfolioID   string
	ValuationDate string
	Status        string
	InputURI      string
	OutputURI     string
}

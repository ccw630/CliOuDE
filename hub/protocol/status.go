package protocol

type RunStatus byte

const (
	Running RunStatus = iota
	Preparing
	PrepareError
	Ok
	RuntimeError
)

var statuses = [5]string{
	"Running", "Preparing", "Prepare Error", "Ok", "Runtime Error",
}

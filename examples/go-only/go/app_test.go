package trace

import "testing"

// REQ-GO-001
// go requirement doc
func TestGoRequirement(t *testing.T) {
	if GoFeatureImpl() == "" {
		t.Fatal("expected traced implementation output")
	}
}

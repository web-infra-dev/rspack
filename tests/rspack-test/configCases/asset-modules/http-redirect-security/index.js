// the whole file is not gonna run, as the compilation should fail
// import "http://localhost:9991/redirect-to-allowed"

// Test disallowed redirect (should fail)
import "http://localhost:9991/redirect-to-disallowed"

// Test non-HTTP protocol redirect (should fail)
import "http://localhost:9991/redirect-to-non-http"

// Test too many redirects (should fail)
import "http://localhost:9991/redirect-chain"

throw new Error('should not reach here')

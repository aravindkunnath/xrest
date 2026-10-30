import { describe, it, expect, afterEach } from 'vitest';
import { RequestParams, serializeForBulkEdit } from "@/core/utils/request-param-utils";


describe("Request param utils", function () {
    it("should return valid bulkedit string", shouldReturnValidBulkEditString);
    it("should handle inactive key value pairs", should_handle_inactive_key_value_pairs);
    it("should handle duplicate key value pairs", should_handle_duplicate_key_value_pairs);
});

function shouldReturnValidBulkEditString() {
    const params: RequestParams[] = [{ key: "test", value: "test", isEnabled: true}];
    const result = serializeForBulkEdit(params);
    expect(result).equals("test: test");
}

function should_handle_inactive_key_value_pairs() {
    const params = [{key:"test", value: "test", isEnabled: true}, {key:"tests", value: "test", isEnabled: false}];
    const result = serializeForBulkEdit(params);
    expect(result).equals(`test: test\n// tests: test`);
}

function should_handle_duplicate_key_value_pairs() {
    const params = [{key:"test", value: "test", isEnabled: true}, {key:"test", value: "test", isEnabled: true}];
    const result = serializeForBulkEdit(params);
    expect(result).equals("test: test");
}
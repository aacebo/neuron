const test = require("node:test");
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const source = fs.readFileSync(path.join(__dirname, "console.js"), "utf8");

test("agent socket uses query credentials and sends no authentication command", () => {
    assert.match(
        source,
        /websocketUrl\("\/agents\/connect",\s*\{\s*agent_id:\s*agent\.id,\s*secret:\s*this\.agentSecret,/,
    );
    assert.doesNotMatch(source, /type:\s*["']authenticate["']/);
});

test("embedded console config is consumed with snake_case fields", () => {
    assert.match(source, /config\.tenant_id/);
    assert.match(source, /config\.high_water_cursor/);
    assert.match(source, /config\.reducer_version/);
    assert.doesNotMatch(source, /config\.(tenantId|highWaterCursor|reducerVersion)/);
});

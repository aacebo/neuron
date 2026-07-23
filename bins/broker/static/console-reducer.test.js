const test = require("node:test");
const assert = require("node:assert/strict");
const Reducer = require("./console-reducer.js");

const tenantId = "775a179d-9819-42c0-ab47-637bf96fbf7b";

function event(id, key, data, traceId = "00000000-0000-0000-0000-000000000100", createdAt) {
    return {
        id,
        tenant_id: tenantId,
        trace_id: traceId,
        key,
        data,
        created_at: createdAt || `2026-07-23T00:00:0${Number(id.slice(-1)) || 0}.000Z`,
    };
}

function actor(id, name, status, skills = []) {
    return {
        id,
        tenant_id: tenantId,
        role: "agent",
        name,
        status,
        instances: status === "online" ? 1 : 0,
        description: `${name} description`,
        skills,
        metadata: {},
        created_at: "2026-07-23T00:00:00.000Z",
        updated_at: "2026-07-23T00:00:00.000Z",
    };
}

test("actor events update status without duplicating the event", () => {
    const state = Reducer.createState(tenantId);
    const id = "00000000-0000-0000-0000-000000000001";
    const create = event(
        "00000000-0000-0000-0000-000000000011",
        "actor.create",
        { type: "actor", actor: actor(id, "calendar", "offline") },
    );
    const update = event(
        "00000000-0000-0000-0000-000000000012",
        "actor.update",
        { type: "actor", actor: actor(id, "calendar", "online") },
    );

    assert.equal(Reducer.reduceEvent(state, create), true);
    assert.equal(Reducer.reduceEvent(state, update), true);
    assert.equal(Reducer.reduceEvent(state, update), false);
    assert.equal(state.actors.get(id).status, "online");
    assert.equal(state.events.length, 2);
});

test("agents and chat membership produce client-side topology", () => {
    const state = Reducer.createState(tenantId);
    const first = "00000000-0000-0000-0000-000000000001";
    const second = "00000000-0000-0000-0000-000000000002";
    const chat_id = "00000000-0000-0000-0000-000000000030";
    const chat_members = event("00000000-0000-0000-0000-000000000023", "chat.members", {
        type: "chat_members",
        chat_id,
        actor_ids: [first, second],
    });
    Reducer.reduceAll(state, [
        event("00000000-0000-0000-0000-000000000021", "actor.create", {
            type: "actor",
            actor: actor(first, "expense", "online", [{ name: "receipts", display_name: "Receipts" }]),
        }),
        event("00000000-0000-0000-0000-000000000022", "actor.create", {
            type: "actor",
            actor: actor(second, "review", "offline", [{ name: "review", display_name: "Review" }]),
        }),
        chat_members,
    ]);

    const topology = Reducer.topology(state);
    const broker = topology.find((element) => element.data.kind === "broker");
    const broker_edges = topology.filter((element) => element.data.kind === "routes_to");
    assert.equal(broker.data.id, "broker_root");
    assert.equal(broker_edges.length, 2);
    assert(broker_edges.every((element) => element.data.source === broker.data.id));
    assert.deepEqual(
        broker_edges.map((element) => element.data.target).sort(),
        [first, second].sort(),
    );
    assert.equal(topology.some((element) => element.data.kind === "skill"), false);
    assert.equal(topology.some((element) => element.data.kind === "has_skill"), false);
    assert(topology.some((element) => element.data.kind === "co_selected" && element.data.weight === 1));
    assert.deepEqual(Reducer.route_agent_ids(state, chat_members).sort(), [first, second].sort());

    const continuation = event("00000000-0000-0000-0000-000000000024", "message.create", {
        type: "message",
        message: { chat: { id: chat_id } },
    });
    assert.deepEqual(Reducer.route_agent_ids(state, continuation).sort(), [first, second].sort());
});

test("out-of-order and unknown events remain visible in their trace", () => {
    const state = Reducer.createState(tenantId);
    const traceId = "00000000-0000-0000-0000-000000000200";
    const later = event(
        "00000000-0000-0000-0000-000000000042",
        "custom.finished",
        { type: "unknown", value: 2 },
        traceId,
        "2026-07-23T00:00:02.000Z",
    );
    const earlier = event(
        "00000000-0000-0000-0000-000000000041",
        "custom.started",
        { type: "unknown", value: 1 },
        traceId,
        "2026-07-23T00:00:01.000Z",
    );

    Reducer.reduceEvent(state, later);
    Reducer.reduceEvent(state, earlier);
    const trace = Reducer.traces(state)[0];
    assert.deepEqual(
        trace.events.map((item) => item.key),
        ["custom.started", "custom.finished"],
    );
});

test("cursor comparison uses timestamp then event id", () => {
    const target = {
        created_at: "2026-07-23T00:00:01.000Z",
        id: "00000000-0000-0000-0000-000000000010",
    };
    assert.equal(Reducer.cursorAtOrAfter(target, target), true);
    assert.equal(
        Reducer.cursorAtOrAfter(
            {
                created_at: target.created_at,
                id: "00000000-0000-0000-0000-000000000009",
            },
            target,
        ),
        false,
    );
});

test("continuation traces remain in one conversation", () => {
    const state = Reducer.createState(tenantId);
    const chatId = "00000000-0000-0000-0000-000000000300";
    const firstTrace = "00000000-0000-0000-0000-000000000301";
    const secondTrace = "00000000-0000-0000-0000-000000000302";
    Reducer.reduceAll(state, [
        event(
            "00000000-0000-0000-0000-000000000051",
            "message.inbound",
            { type: "inbound_message", message: { content: [{ type: "text", text: "first" }] } },
            firstTrace,
            "2026-07-23T00:00:01.000Z",
        ),
        event(
            "00000000-0000-0000-0000-000000000052",
            "chat.create",
            { type: "chat", chat: { id: chatId } },
            firstTrace,
            "2026-07-23T00:00:02.000Z",
        ),
        event(
            "00000000-0000-0000-0000-000000000053",
            "message.inbound",
            {
                type: "inbound_message",
                message: { chat_id: chatId, content: [{ type: "text", text: "second" }] },
            },
            secondTrace,
            "2026-07-23T00:00:03.000Z",
        ),
        event(
            "00000000-0000-0000-0000-000000000054",
            "message.create",
            { type: "message", message: { chat: { id: chatId } } },
            secondTrace,
            "2026-07-23T00:00:04.000Z",
        ),
    ]);

    const traces = Reducer.traces(state);
    assert.equal(Reducer.conversations(traces).length, 1);
    assert.equal(Reducer.traceChatId(traces.find((trace) => trace.id === firstTrace)), chatId);
    assert.deepEqual(
        Reducer.conversationEvents(traces, chatId, null).map((item) => item.key),
        ["message.inbound", "chat.create", "message.inbound", "message.create"],
    );
});

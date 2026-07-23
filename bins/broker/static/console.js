(function () {
    "use strict";

    const Reducer = window.NeuronConsoleReducer;

    function readConfig() {
        return JSON.parse(document.getElementById("console-config").textContent);
    }

    function websocketUrl(path, params) {
        const url = new URL(path, window.location.href);
        url.protocol = window.location.protocol === "https:" ? "wss:" : "ws:";
        for (const [key, value] of Object.entries(params || {})) {
            if (value !== null && value !== undefined && value !== "") url.searchParams.set(key, value);
        }
        return url.toString();
    }

    async function openDatabase(config) {
        const name = `neuron-console-${config.reducerVersion}-${config.tenantId}`;
        const activeKey = "neuron-console-active-cache";
        const previous = localStorage.getItem(activeKey);
        if (previous && previous !== name) {
            await new Promise((resolve) => {
                const request = indexedDB.deleteDatabase(previous);
                request.onsuccess = resolve;
                request.onerror = resolve;
                request.onblocked = resolve;
            });
        }
        localStorage.setItem(activeKey, name);

        return new Promise((resolve, reject) => {
            const request = indexedDB.open(name, 1);
            request.onupgradeneeded = () => {
                const database = request.result;
                if (!database.objectStoreNames.contains("events")) {
                    database.createObjectStore("events", {
                        keyPath: ["tenant_id", "created_at", "id"],
                    });
                }
            };
            request.onsuccess = () => resolve(request.result);
            request.onerror = () => reject(request.error);
        });
    }

    function databaseRequest(database, mode, operation) {
        return new Promise((resolve, reject) => {
            const transaction = database.transaction("events", mode);
            const request = operation(transaction.objectStore("events"));
            request.onsuccess = () => resolve(request.result);
            request.onerror = () => reject(request.error);
        });
    }

    function createConsole() {
        const config = readConfig();
        const state = Reducer.createState(config.tenantId);

        return {
            config,
            state,
            database: null,
            eventSocket: null,
            agentSocket: null,
            graph: null,
            reconnectTimer: null,
            reconnectDelay: 800,

            streamStatus: "starting",
            agentStatus: "disconnected",
            tab: "compose",
            senderMode: "user",
            agents: [],
            traceItems: [],
            selectedTraceId: null,
            activeChatId: null,
            selectedEvent: null,
            selectedEntity: null,
            pendingTraceId: null,
            draftTraceId: crypto.randomUUID(),
            error: "",
            notice: "",

            userExternalId: "console-user",
            userName: "Console User",
            selectedAgentId: "",
            agentSecret: "",
            subject: "",
            messageText: "",
            chatDraft: "",
            contentBlocks: [],
            metadataText: "{}",
            rawMode: false,
            rawPayload: "",
            traceFilter: "",
            keyFilter: "",
            traceAgentFilter: "",
            traceAfter: "",
            traceBefore: "",
            inspectorMode: "tree",
            inspectorSearch: "",
            expandedJsonPaths: ["$", "data"],

            async init() {
                try {
                    this.database = await openDatabase(config);
                    const cached = await databaseRequest(this.database, "readonly", (store) => store.getAll());
                    Reducer.reduceAll(state, cached);
                    this.refreshDerived();
                    this.openEventStream();
                } catch (error) {
                    this.error = `Local event cache unavailable: ${error.message}`;
                    this.openEventStream();
                }
                window.addEventListener("beforeunload", () => {
                    this.eventSocket?.close();
                    this.agentSocket?.close();
                });
            },

            openEventStream() {
                clearTimeout(this.reconnectTimer);
                const cursor = Reducer.latestCursor(state);
                const url = websocketUrl("/console/connect", {
                    after_at: cursor?.created_at,
                    after_id: cursor?.id,
                });
                this.streamStatus = "connecting";
                const socket = new WebSocket(url);
                this.eventSocket = socket;

                socket.onopen = () => {
                    this.reconnectDelay = 800;
                    this.streamStatus = Reducer.cursorAtOrAfter(cursor, config.highWaterCursor)
                        ? "live"
                        : "replaying";
                };
                socket.onmessage = (message) => {
                    try {
                        const event = JSON.parse(message.data);
                        this.ingestEvent(event);
                        if (Reducer.cursorAtOrAfter(Reducer.latestCursor(state), config.highWaterCursor)) {
                            this.streamStatus = "live";
                        }
                    } catch (error) {
                        this.error = `Invalid event frame: ${error.message}`;
                    }
                };
                socket.onerror = () => {
                    this.streamStatus = "degraded";
                };
                socket.onclose = () => {
                    if (this.eventSocket !== socket) return;
                    this.streamStatus = "offline";
                    this.reconnectTimer = setTimeout(() => this.openEventStream(), this.reconnectDelay);
                    this.reconnectDelay = Math.min(this.reconnectDelay * 1.8, 8000);
                };
            },

            ingestEvent(event) {
                if (!Reducer.reduceEvent(state, event)) return;
                if (this.database) {
                    databaseRequest(this.database, "readwrite", (store) => store.put(event)).catch(() => {});
                }
                if (this.pendingTraceId === event.trace_id) {
                    this.selectedTraceId = event.trace_id;
                    this.selectedEvent = event;
                    this.pendingTraceId = null;
                    this.tab = "chat";
                }
                if (this.selectedTraceId === event.trace_id) {
                    this.activeChatId = this.eventChatId(event) || this.activeChatId;
                }
                this.refreshDerived();
            },

            refreshDerived() {
                this.agents = Array.from(state.actors.values())
                    .filter((actor) => actor.role === "agent")
                    .sort((left, right) => left.name.localeCompare(right.name));
                this.traceItems = Reducer.traces(state);
                if (!this.selectedAgentId && this.agents.length) this.selectedAgentId = this.agents[0].id;
                if (this.selectedTraceId && !state.traces.has(this.selectedTraceId)) this.selectedTraceId = null;
                if (this.graph) queueMicrotask(() => this.renderGraph());
            },

            selectTab(tab) {
                this.tab = tab;
                if (tab === "topology") setTimeout(() => this.ensureGraph(), 0);
            },

            ensureGraph() {
                if (this.graph || !this.$refs.topology || !window.cytoscape) return;
                this.graph = window.cytoscape({
                    container: this.$refs.topology,
                    elements: [],
                    minZoom: 0.35,
                    maxZoom: 2.4,
                    style: [
                        {
                            selector: "node",
                            style: {
                                "font-family": "Inter, ui-sans-serif, system-ui",
                                "font-size": 11,
                                "text-valign": "bottom",
                                "text-margin-y": 8,
                                "text-wrap": "wrap",
                                "text-max-width": 112,
                                color: "#a9b2c3",
                                label: "data(label)",
                                "background-color": "#141922",
                                "border-width": 1,
                                "border-color": "#394253",
                            },
                        },
                        {
                            selector: "node.agent",
                            style: {
                                width: 46,
                                height: 46,
                                "background-color": "#111722",
                                "border-width": 2,
                                "border-color": "#596579",
                            },
                        },
                        {
                            selector: "node.agent.online",
                            style: {
                                "border-color": "#34d399",
                            },
                        },
                        {
                            selector: "node.skill",
                            style: {
                                width: 18,
                                height: 18,
                                "font-size": 9,
                                "background-color": "#17222b",
                                "border-color": "#2e7181",
                                color: "#718096",
                            },
                        },
                        {
                            selector: "edge",
                            style: {
                                width: 1,
                                "line-color": "#2b3442",
                                "curve-style": "bezier",
                                opacity: 0.7,
                            },
                        },
                        {
                            selector: "edge.has-skill",
                            style: {
                                "line-style": "dashed",
                                "line-color": "#244b57",
                                opacity: 0.5,
                            },
                        },
                        {
                            selector: "edge.co-selected",
                            style: {
                                width: "mapData(weight, 1, 8, 1.5, 6)",
                                "line-color": "#557087",
                                label: "data(label)",
                                "font-size": 9,
                                color: "#68768a",
                            },
                        },
                        {
                            selector: ".trace-match",
                            style: {
                                "border-color": "#67e8f9",
                                "border-width": 3,
                            },
                        },
                        {
                            selector: ":selected",
                            style: {
                                "border-color": "#f8fafc",
                                "border-width": 2,
                            },
                        },
                    ],
                });
                this.graph.on("tap", "node, edge", (event) => {
                    const data = event.target.data();
                    this.selectedEntity = data.actor || data.skill || {
                        kind: data.kind,
                        weight: data.weight,
                        source: data.source,
                        target: data.target,
                    };
                });
                this.renderGraph();
            },

            renderGraph() {
                if (!this.graph) return;
                this.graph.elements().remove();
                this.graph.add(Reducer.topology(state));
                this.applyTraceHighlight();
                const layout = this.graph.layout({
                    name: "cose",
                    animate: false,
                    fit: true,
                    padding: 60,
                    nodeRepulsion: 18000,
                    idealEdgeLength: 145,
                    componentSpacing: 110,
                    gravity: 0.16,
                    nodeOverlap: 24,
                });
                layout.on("layoutstop", () => this.graph.fit(undefined, 60));
                layout.run();
            },

            fitGraph() {
                this.graph?.fit(undefined, 60);
            },

            applyTraceHighlight() {
                if (!this.graph) return;
                this.graph.elements().removeClass("trace-match");
                const trace = this.traceItems.find((item) => item.id === this.selectedTraceId);
                for (const id of trace?.agentIds || []) this.graph.getElementById(id).addClass("trace-match");
            },

            selectAgent(agentId) {
                if (this.agentSocket) this.disconnectAgent();
                this.selectedAgentId = agentId;
                this.agentSecret = sessionStorage.getItem(`neuron-agent-secret:${agentId}`) || "";
                this.selectedEntity = this.agents.find((agent) => agent.id === agentId) || null;
            },

            connectAgent() {
                this.error = "";
                const agent = this.agents.find((item) => item.id === this.selectedAgentId);
                if (!agent || !this.agentSecret) {
                    this.error = "Select an agent and enter its secret.";
                    return;
                }
                this.disconnectAgent();
                sessionStorage.setItem(`neuron-agent-secret:${agent.id}`, this.agentSecret);
                this.agentStatus = "connecting";
                const socket = new WebSocket(websocketUrl("/agents/connect"));
                this.agentSocket = socket;
                socket.onopen = () => {
                    socket.send(
                        JSON.stringify({
                            type: "authenticate",
                            agent_id: agent.id,
                            secret: this.agentSecret,
                        }),
                    );
                };
                socket.onmessage = (message) => {
                    try {
                        const event = JSON.parse(message.data);
                        this.ingestEvent(event);
                        if (
                            event.key === "actor.update" &&
                            event.data?.actor?.id === agent.id &&
                            event.data.actor.status === "online"
                        ) {
                            this.agentStatus = "connected";
                            this.notice = `${agent.name} connected`;
                        }
                    } catch (error) {
                        this.error = `Invalid agent event: ${error.message}`;
                    }
                };
                socket.onclose = (event) => {
                    if (this.agentSocket !== socket) return;
                    this.agentSocket = null;
                    this.agentStatus = "disconnected";
                    if (event.code !== 1000 && event.reason) this.error = event.reason;
                };
                socket.onerror = () => {
                    this.agentStatus = "error";
                };
            },

            disconnectAgent() {
                const socket = this.agentSocket;
                this.agentSocket = null;
                if (socket && socket.readyState < WebSocket.CLOSING) socket.close(1000, "console disconnect");
                this.agentStatus = "disconnected";
            },

            structuredPayload() {
                const metadata = this.parseMetadata();
                const content = this.structuredContent();
                if (this.senderMode === "agent") {
                    return {
                        type: "message_send",
                        trace_id: this.draftTraceId,
                        subject: this.subject || null,
                        content,
                        metadata,
                    };
                }
                return {
                    tenant_id: config.tenantId,
                    subject: this.subject || null,
                    content,
                    metadata,
                    from: {
                        id: this.userExternalId,
                        name: this.userName,
                    },
                };
            },

            structuredContent() {
                const content = [];
                if (this.messageText.trim()) content.push({ type: "text", text: this.messageText });
                for (const block of this.contentBlocks) {
                    if (block.type === "text") {
                        if (block.value.trim()) content.push({ type: "text", text: block.value });
                    } else if (block.type === "json") {
                        content.push({ type: "json", json: JSON.parse(block.value) });
                    } else if (block.type === "file") {
                        const uri = new URL(block.value).toString();
                        content.push({
                            type: "file",
                            ...(block.name.trim() ? { name: block.name.trim() } : {}),
                            uri,
                        });
                    }
                }
                return content;
            },

            addContentBlock(type = "json") {
                this.contentBlocks.push({
                    id: crypto.randomUUID(),
                    type,
                    name: "",
                    value: type === "json" ? "{}" : "",
                });
            },

            removeContentBlock(id) {
                this.contentBlocks = this.contentBlocks.filter((block) => block.id !== id);
            },

            parseMetadata() {
                const metadata = JSON.parse(this.metadataText || "{}");
                if (!metadata || Array.isArray(metadata) || typeof metadata !== "object") {
                    throw new Error("Metadata must be a JSON object.");
                }
                return metadata;
            },

            previewPayload() {
                try {
                    return this.rawMode ? JSON.parse(this.rawPayload || "{}") : this.structuredPayload();
                } catch (error) {
                    return { error: error.message };
                }
            },

            toggleRawMode() {
                if (!this.rawMode) this.rawPayload = JSON.stringify(this.structuredPayload(), null, 2);
                this.rawMode = !this.rawMode;
            },

            formatPayload() {
                try {
                    this.rawPayload = JSON.stringify(JSON.parse(this.rawPayload), null, 2);
                    this.error = "";
                } catch (error) {
                    this.error = `Invalid JSON: ${error.message}`;
                }
            },

            async copyPayload() {
                await navigator.clipboard.writeText(JSON.stringify(this.previewPayload(), null, 2));
                this.notice = "Payload copied";
            },

            async sendMessage() {
                this.error = "";
                this.notice = "";
                try {
                    const payload = this.rawMode ? JSON.parse(this.rawPayload) : this.structuredPayload();
                    await this.dispatchMessage(payload);
                } catch (error) {
                    this.error = error.message;
                }
            },

            async sendChatMessage() {
                this.error = "";
                this.notice = "";
                const text = this.chatDraft.trim();
                if (!text) return;
                try {
                    const chatId = this.currentChatId();
                    if (!chatId) throw new Error("Wait for the conversation to be created before replying.");
                    await this.dispatchMessage({
                        chat_id: chatId,
                        subject: null,
                        content: [{ type: "text", text }],
                        metadata: {},
                    });
                    this.chatDraft = "";
                } catch (error) {
                    this.error = error.message;
                }
            },

            async dispatchMessage(input) {
                let payload = input;
                if (!payload.content?.length) throw new Error("At least one content block is required.");
                const traceId = payload.trace_id || this.draftTraceId;
                this.activeChatId = payload.chat_id || null;
                this.pendingTraceId = traceId;
                this.selectedTraceId = traceId;
                this.tab = "chat";

                try {
                    if (this.senderMode === "agent") {
                        if (!this.agentSocket || this.agentSocket.readyState !== WebSocket.OPEN || this.agentStatus !== "connected") {
                            throw new Error("Connect the selected agent before sending.");
                        }
                        payload = { ...payload, type: "message_send", trace_id: traceId };
                        this.agentSocket.send(JSON.stringify(payload));
                    } else {
                        payload = {
                            ...payload,
                            tenant_id: config.tenantId,
                            from: payload.from || {
                                id: this.userExternalId,
                                name: this.userName,
                            },
                        };
                        const response = await fetch("/messages", {
                            method: "POST",
                            headers: {
                                "Content-Type": "application/json",
                                "X-Request-ID": traceId,
                            },
                            body: JSON.stringify(payload),
                        });
                        if (!response.ok) throw new Error(`${response.status} ${await response.text()}`);
                    }
                } catch (error) {
                    if (this.pendingTraceId === traceId) this.pendingTraceId = null;
                    throw error;
                }

                this.draftTraceId = crypto.randomUUID();
                this.notice = `Sent · ${traceId.slice(0, 8)}`;
            },

            filteredTraces() {
                const query = this.traceFilter.trim().toLowerCase();
                const key = this.keyFilter.trim().toLowerCase();
                const after = this.traceAfter ? Date.parse(this.traceAfter) : null;
                const before = this.traceBefore ? Date.parse(this.traceBefore) : null;
                return this.traceItems.filter((trace) => {
                    if (key && !trace.keys.some((value) => value.toLowerCase().includes(key))) return false;
                    if (this.traceAgentFilter && !trace.agentIds.includes(this.traceAgentFilter)) return false;
                    if (after !== null && Date.parse(trace.updatedAt) < after) return false;
                    if (before !== null && Date.parse(trace.startedAt) > before) return false;
                    if (!query) return true;
                    return (
                        trace.id.toLowerCase().includes(query) ||
                        JSON.stringify(trace.events).toLowerCase().includes(query)
                    );
                });
            },

            chatConversations() {
                return Reducer.conversations(this.traceItems);
            },

            currentChatTrace() {
                return this.traceItems.find((trace) => trace.id === this.selectedTraceId) || null;
            },

            currentChatId() {
                return this.activeChatId || this.traceChatId(this.currentChatTrace());
            },

            eventChatId(event) {
                return Reducer.eventChatId(event);
            },

            traceChatId(trace) {
                return Reducer.traceChatId(trace);
            },

            conversationEvents() {
                const chatId = this.currentChatId();
                return Reducer.conversationEvents(this.traceItems, chatId, this.selectedTraceId);
            },

            chatMessages() {
                const inboundFingerprints = new Set();
                const messages = [];
                for (const event of this.conversationEvents()) {
                    const message =
                        event.key === "message.inbound"
                            ? event.data?.message
                            : event.key === "message.create"
                              ? event.data?.message
                              : null;
                    if (!message) continue;
                    const actor = message.sent_by || message.created_by || null;
                    const fingerprint = `${actor?.id || "unknown"}:${JSON.stringify(message.content || [])}`;
                    if (event.key === "message.inbound") {
                        inboundFingerprints.add(fingerprint);
                        messages.push({ event, actor, content: message.content || [] });
                    } else if (!inboundFingerprints.has(fingerprint)) {
                        messages.push({ event, actor, content: message.content || [] });
                    }
                }
                return messages;
            },

            chatActivity() {
                return this.conversationEvents().filter(
                    (event) => event.key !== "message.inbound" && event.key !== "message.create",
                );
            },

            selectChatTrace(traceId) {
                const trace = this.traceItems.find((item) => item.id === traceId);
                if (trace) this.selectTrace(trace);
            },

            contentText(block) {
                if (block.type === "text") return block.text;
                if (block.type === "json") return JSON.stringify(block.json, null, 2);
                return block.name || block.uri || block.base64 || "file";
            },

            selectTrace(trace) {
                this.selectedTraceId = trace.id;
                this.activeChatId = this.traceChatId(trace);
                this.selectedEvent = trace.events[trace.events.length - 1] || null;
                this.selectedEntity = null;
                this.applyTraceHighlight();
            },

            currentTraceEvents() {
                return this.traceItems.find((trace) => trace.id === this.selectedTraceId)?.events || [];
            },

            traceLabel(trace) {
                const inbound = trace.events.find((event) => event.key === "message.inbound");
                const text = inbound?.data?.message?.content?.find((item) => item.type === "text")?.text;
                return text || trace.keys[0] || "trace";
            },

            eventDelta(event, index) {
                if (index === 0) return "0 ms";
                const previous = this.currentTraceEvents()[index - 1];
                const delta = Date.parse(event.created_at) - Date.parse(previous.created_at);
                return `${Math.max(0, delta)} ms`;
            },

            inspectEvent(event) {
                this.selectedEvent = event;
                this.selectedEntity = null;
                this.expandedJsonPaths = ["$", "data"];
            },

            inspectorValue() {
                return this.selectedEvent || this.selectedEntity || null;
            },

            inspectorJson() {
                const value = this.inspectorValue();
                if (!value) return "";
                const json = JSON.stringify(value, null, 2);
                const search = this.inspectorSearch.trim().toLowerCase();
                return search && !json.toLowerCase().includes(search) ? "No matching JSON value." : json;
            },

            inspectorTree() {
                const rows = [];
                const walk = (value, path, depth) => {
                    const rowPath = path || "$";
                    const expandable = value !== null && typeof value === "object";
                    const entries = expandable
                        ? Array.isArray(value)
                            ? value.map((item, index) => [index, item])
                            : Object.entries(value)
                        : [];
                    const expanded = this.expandedJsonPaths.includes(rowPath);
                    rows.push({
                        path: rowPath,
                        key: path ? path.split(".").at(-1) : "$",
                        value: expandable ? this.objectPreview(value) : this.primitivePreview(value),
                        valueType: expandable ? "object" : value === null ? "null" : typeof value,
                        depth,
                        expandable,
                        expanded,
                        raw: value,
                    });
                    if (expandable && (expanded || this.inspectorSearch.trim())) {
                        for (const [key, child] of entries) {
                            const childPath = path
                                ? Array.isArray(value)
                                    ? `${path}[${key}]`
                                    : `${path}.${key}`
                                : String(key);
                            walk(child, childPath, depth + 1);
                        }
                    }
                };
                const value = this.inspectorValue();
                if (value) walk(value, "", 0);
                const search = this.inspectorSearch.trim().toLowerCase();
                return search
                    ? rows.filter((row) => `${row.path} ${row.value}`.toLowerCase().includes(search))
                    : rows;
            },

            toggleJsonPath(path) {
                this.expandedJsonPaths = this.expandedJsonPaths.includes(path)
                    ? this.expandedJsonPaths.filter((value) => value !== path)
                    : [...this.expandedJsonPaths, path];
            },

            objectPreview(value) {
                if (Array.isArray(value)) return `Array(${value.length})`;
                const keys = Object.keys(value);
                if (!keys.length) return "{}";
                const preview = keys
                    .slice(0, 2)
                    .map((key) => `${key}: ${this.primitivePreview(value[key], true)}`)
                    .join(", ");
                return `{${preview}${keys.length > 2 ? ", …" : ""}}`;
            },

            primitivePreview(value, compact = false) {
                if (value === null) return "null";
                if (typeof value === "string") {
                    const text = compact && value.length > 26 ? `${value.slice(0, 26)}…` : value;
                    return JSON.stringify(text);
                }
                if (typeof value === "undefined") return "undefined";
                if (typeof value === "object") return Array.isArray(value) ? `Array(${value.length})` : "{…}";
                return String(value);
            },

            async copyInspector() {
                const value = this.inspectorValue();
                if (!value) return;
                await navigator.clipboard.writeText(JSON.stringify(value, null, 2));
                this.notice = "Inspector JSON copied";
            },

            async copyTraceId() {
                if (!this.selectedTraceId) return;
                await navigator.clipboard.writeText(this.selectedTraceId);
                this.notice = "Trace ID copied";
            },

            compactId(value) {
                return value ? `${value.slice(0, 8)}…${value.slice(-4)}` : "—";
            },

            formatTime(value) {
                if (!value) return "—";
                return new Intl.DateTimeFormat(undefined, {
                    hour: "2-digit",
                    minute: "2-digit",
                    second: "2-digit",
                    fractionalSecondDigits: 3,
                }).format(new Date(value));
            },

            formatJson(value) {
                return JSON.stringify(value, null, 2);
            },

            async resetLocalState() {
                this.eventSocket?.close();
                this.agentSocket?.close();
                this.database?.close();
                await new Promise((resolve, reject) => {
                    const request = indexedDB.deleteDatabase(
                        `neuron-console-${config.reducerVersion}-${config.tenantId}`,
                    );
                    request.onsuccess = resolve;
                    request.onerror = () => reject(request.error);
                    request.onblocked = resolve;
                });
                window.location.reload();
            },
        };
    }

    document.addEventListener("alpine:init", () => {
        window.Alpine.data("neuronConsole", createConsole);
    });
})();

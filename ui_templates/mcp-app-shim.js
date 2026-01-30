/**
 * Minimal MCP Apps protocol shim (vanilla JS).
 *
 * Implements the postMessage JSON-RPC 2.0 protocol between an MCP App
 * iframe and the host (VS Code, Claude Desktop, etc.).
 *
 * Protocol spec: https://modelcontextprotocol.io/docs/extensions/apps
 * Protocol version: 2026-01-26
 */
class McpApp {
  #nextId = 1;
  #pending = new Map();
  #name;
  #version;

  constructor(name, version) {
    this.#name = name;
    this.#version = version;
    this.ontoolresult = null;
    this.ontoolinput = null;
    this.ontoolinputpartial = null;
    this.onhostcontext = null;
    this.ontoolcancelled = null;
    window.addEventListener('message', (e) => this.#handleMessage(e));
  }

  async connect() {
    const result = await this.#request('ui/initialize', {
      appInfo: { name: this.#name, version: this.#version },
      appCapabilities: {},
      protocolVersion: '2026-01-26',
    });
    this.#notify('ui/notifications/initialized', {});
    return result;
  }

  async callServerTool(name, args) {
    return this.#request('tools/call', { name: name, arguments: args || {} });
  }

  sendSizeChanged(width, height) {
    this.#notify('ui/notifications/size-changed', { width: width, height: height });
  }

  async sendMessage(content) {
    return this.#request('ui/message', { role: 'user', content: content });
  }

  #handleMessage(event) {
    var msg = event.data;
    if (!msg || msg.jsonrpc !== '2.0') return;

    // Response to our request
    if (msg.id != null && this.#pending.has(msg.id)) {
      var handler = this.#pending.get(msg.id);
      this.#pending.delete(msg.id);
      if (msg.error) handler.reject(msg.error);
      else handler.resolve(msg.result);
      return;
    }

    // Notification from host
    if (msg.method === 'ui/notifications/tool-result') {
      if (this.ontoolresult) this.ontoolresult(msg.params);
    } else if (msg.method === 'ui/notifications/tool-input') {
      if (this.ontoolinput) this.ontoolinput(msg.params);
    } else if (msg.method === 'ui/notifications/tool-input-partial') {
      if (this.ontoolinputpartial) this.ontoolinputpartial(msg.params);
    } else if (msg.method === 'ui/notifications/host-context-changed') {
      if (this.onhostcontext) this.onhostcontext(msg.params);
    } else if (msg.method === 'ui/notifications/tool-cancelled') {
      if (this.ontoolcancelled) this.ontoolcancelled(msg.params);
    }
  }

  #request(method, params) {
    var self = this;
    return new Promise(function(resolve, reject) {
      var id = self.#nextId++;
      self.#pending.set(id, { resolve: resolve, reject: reject });
      window.parent.postMessage({ jsonrpc: '2.0', id: id, method: method, params: params }, '*');
    });
  }

  #notify(method, params) {
    window.parent.postMessage({ jsonrpc: '2.0', method: method, params: params }, '*');
  }
}

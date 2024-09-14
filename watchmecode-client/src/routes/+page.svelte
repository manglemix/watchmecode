<script lang="ts">
    import CodeMirror from "svelte-codemirror-editor";
    import { defaultKeymap } from "@codemirror/commands";
    import { keymap } from "@codemirror/view";
	import { onDestroy } from "svelte";
	import { browser } from "$app/environment";

    async function delay(ms: number): Promise<void> {
        return new Promise(resolve => setTimeout(resolve, ms));
    }

    let value = "";
    let lastValue = "";
    let username = "";
    let oldUsername = "";
    let host_code = "";
    let showing_host_code = false;
    let websocket: WebSocket | null = null;
    let lastUpdate = 0;

    if (browser) {
        const urlParams = new URLSearchParams(window.location.search);
        let host = urlParams.get('host') ?? "ws://127.0.0.1";
        if (host.endsWith("/")) {
            host = host.slice(0, -1);
        }
        function openWebSocket() {
            websocket = new WebSocket(`${host}/code/`);
            websocket.onmessage = (event) => {
                host_code = event.data;
                lastUpdate = 0;
            };
            websocket.onclose = (_) => {
                if (websocket === null) {
                    return;
                }
                websocket = null;
                delay(1000).then(openWebSocket);
            };
            websocket.onerror = (_) => {
                if (websocket === null) {
                    return;
                }
                websocket = null;
                delay(1000).then(openWebSocket);
            };
            websocket.onopen = () => {
                onNameChanged();
                if (value != "") {
                    websocket!.send(`CODE: ${value}`);
                }
            };
        }
        openWebSocket();

        async function postCode() {
            if (lastValue == value) {
                return;
            }
            lastValue = value;
            websocket!.send(`CODE: ${value}`);
        }

        const interval = setInterval(postCode, 2000);
        const interval2 = setInterval(() => {
            lastUpdate++;
        }, 1000);

        onDestroy(() => {
            clearInterval(interval);
            clearInterval(interval2);
        });
    }

    function onNameChanged() {
        if (!/^[a-zA-Z0-9]+$/.test(username)) {
            username = oldUsername;
            return;
        }

        oldUsername = username;
        websocket!.send(`NAME: ${username}`);
    }
</script>

<svelte:head>
    <title>WatchMeCode</title>
</svelte:head>

<label for=name>Name:</label>
<input id=name bind:value={username} on:change={onNameChanged}>
<hr>
<CodeMirror bind:value basic={false} extensions={[keymap.of([
    ...defaultKeymap,
  ])
]}/>
<hr>
{#if showing_host_code}
    <button on:click={() => {showing_host_code = false}}>Hide host code</button>
    <br>
    <p>Last updated: {lastUpdate} seconds ago</p>
    {#if host_code == ""}
        <i>No code yet from host</i>
    {:else}
        <code>{host_code}</code>
    {/if}
{:else}
    <button on:click={() => {showing_host_code = true}}>Show host code</button>
{/if}
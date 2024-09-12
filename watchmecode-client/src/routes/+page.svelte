<script lang="ts">
    import CodeMirror from "svelte-codemirror-editor";
    import { defaultKeymap } from "@codemirror/commands";
    import { keymap } from "@codemirror/view";
	import { onDestroy } from "svelte";
	import { browser } from "$app/environment";

    function generateRandomString(length: number) {
        const characters = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789';
        let result = '';
        const charactersLength = characters.length;
        for (let i = 0; i < length; i++) {
            result += characters.charAt(Math.floor(Math.random() * charactersLength));
        }
        return result;
    }
    
    let value = "";

    if (browser) {
        let lastValue = "";
        let id = generateRandomString(12);

        const urlParams = new URLSearchParams(window.location.search);
        const host = urlParams.get('host') ?? "127.0.0.1";

        async function postCode() {
            if (lastValue == value) {
                return;
            }
            lastValue = value;
            fetch(`http://${host}/code/${id}`, {
                method: "POST",
                headers: {
                    "Content-Type": "text/plain",
                },
                body: value,
                mode: "no-cors",
            });
        }

        const interval = setInterval(postCode, 2000);

        onDestroy(() => {
            clearInterval(interval);
        });
    }

</script>

<CodeMirror bind:value basic={false} extensions={[keymap.of([
    ...defaultKeymap,
  ])
]}/>
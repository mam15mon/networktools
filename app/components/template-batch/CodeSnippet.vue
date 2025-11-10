<template>
	<div class="relative group">
		<pre
			class="font-mono text-xs md:text-sm bg-(--ui-bg-muted) border border-(--ui-border) rounded-md px-3 py-3 whitespace-pre-wrap break-words overflow-x-auto"
		><code v-text="code" /></pre>
		<UButton
			variant="ghost"
			size="2xs"
			color="primary"
			class="absolute top-2 right-2 opacity-80 hover:opacity-100"
			@click="copyToClipboard"
		>
			<Icon :name="copied ? 'i-lucide-copy-check' : 'i-lucide-copy'" class="size-4" />
			<span class="sr-only">复制代码</span>
		</UButton>
	</div>
</template>

<script setup lang="ts">
import { ref } from "vue";

const props = defineProps<{
	code: string;
}>();

const copied = ref(false);

async function copyToClipboard() {
	try {
		await navigator.clipboard.writeText(props.code);
		copied.value = true;
		setTimeout(() => {
			copied.value = false;
		}, 2000);
	} catch (error) {
		console.error("复制失败", error);
	}
}
</script>

<template>
	<div v-if="items.length" class="space-y-4">
		<UCard
			v-for="item in limitedItems"
			:key="`config-card-${item.label}`"
			class="bg-(--ui-bg)"
		>
			<template #header>
				<div class="flex items-center justify-between gap-3">
					<button class="flex flex-1 items-center gap-2 text-left" type="button" @click="toggleItem(item.label)">
						<UIcon :name="isOpen(item.label) ? 'i-lucide-chevron-down' : 'i-lucide-chevron-right'" class="size-4 text-(--ui-text-muted)" />
						<div>
							<p class="text-sm font-semibold">{{ item.label }}</p>
							<p class="text-xs text-(--ui-text-muted)">{{ item.lines.length }} 行</p>
						</div>
					</button>
					<UButton
						variant="ghost"
						size="xs"
						icon="i-lucide-copy"
						@click.stop="copyConfig(item.config, item.label)"
					>
						复制
					</UButton>
				</div>
			</template>
			<Transition name="fade-slide">
				<div v-if="isOpen(item.label)" class="pt-2">
					<pre class="bg-(--ui-bg-muted) rounded p-3 text-xs whitespace-pre-wrap">{{ item.config }}</pre>
				</div>
			</Transition>
		</UCard>
		<p v-if="limit && items.length > limit" class="text-xs text-(--ui-text-muted)">
			仅展示前 {{ limit }} 条配置。
		</p>
	</div>
	<div v-else class="rounded-lg border border-dashed border-(--ui-border) p-4 text-sm text-(--ui-text-muted)">
		{{ emptyMessage }}
	</div>
</template>

<script setup lang="ts">
import { computed, ref } from "vue";
import type { PropType } from "vue";

interface ConfigItem {
	label: string
	config: string
}

const props = defineProps({
	items: {
		type: Array as PropType<ConfigItem[]>,
		default: () => []
	},
	limit: {
		type: Number,
		default: 5
	},
	emptyMessage: {
		type: String,
		default: "暂无配置"
	}
});

const toast = useToast();

const limitedItems = computed(() =>
	props.items.slice(0, props.limit).map((item) => ({
		label: item.label,
		config: item.config,
		lines: item.config.split(/\r?\n/)
	}))
);

const openKeys = ref<string[]>([]);

function toggleItem(label: string) {
	const index = openKeys.value.indexOf(label);
	if (index === -1) {
		openKeys.value.push(label);
	} else {
		openKeys.value.splice(index, 1);
	}
}

function isOpen(label: string) {
	return openKeys.value.includes(label);
}

async function copyConfig(content: string, label: string) {
	try {
		await navigator.clipboard.writeText(content);
		toast.add({ title: "复制成功", description: `${label} 配置已复制`, color: "success" });
	} catch (error) {
		toast.add({ title: "复制失败", description: String(error), color: "error" });
	}
}
</script>

<template>
	<div class="px-6 pb-10 pt-12 sm:pt-16 lg:px-10 lg:pt-22 min-h-screen">
		<div class="mx-auto max-w-6xl space-y-6">
			<h2 class="text-3xl font-bold tracking-tight">
				{{ props.title }}
			</h2>
			<p class="text-lg text-(--ui-text-muted) leading-8">
				{{ props.description }}
			</p>
			<div class="min-h-[calc(100vh-12rem)]">
				<slot />
			</div>
		</div>
	</div>
</template>

<script lang="ts" setup>
	const props = defineProps<{
		title: string
		description: string
	}>();
</script>

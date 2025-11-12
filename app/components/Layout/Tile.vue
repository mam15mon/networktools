<template>
	<div class="px-4 pb-10 pt-10 sm:px-6 sm:pt-12 lg:px-8 lg:pb-14 min-h-[100dvh]">
		<div class="mx-auto w-full max-w-screen-xl space-y-6">
			<h2 class="text-3xl font-bold tracking-tight">
				{{ props.title }}
			</h2>
			<p class="text-lg text-(--ui-text-muted) leading-8">
				{{ props.description }}
			</p>
			<div class="min-h-[calc(100dvh-10rem)]">
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

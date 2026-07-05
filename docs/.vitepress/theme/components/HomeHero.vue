<template>
  <div class="-mt-16">
    <Ray class="h-[60vh] pointer-events-none absolute top-0 opacity-[.35] dark:opacity-50" />
    <section class="relative h-screen flex items-center justify-center pt-16 overflow-hidden min-h-[800px]">
      <div class="relative max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 text-center z-10">
        <h1 class="text-5xl md:text-7xl font-bold tracking-tight text-gray-600 dark:text-white mb-6">
          Manage runtimes
          <span class="text-transparent bg-clip-text bg-gradient-to-r from-[#185fa5] to-[#4a90d9]">effortlessly</span>
        </h1>

        <p class="mt-4 max-w-2xl mx-auto text-xl text-gray-500 dark:text-gray-300">
          Install, activate, and switch between multiple versions of PHP, Node.js, Python, MariaDB, and more — from CLI or GUI.
        </p>

        <div class="mt-8 flex flex-col sm:flex-row gap-4 justify-center">
          <a href="/guide/getting-started" class="px-6 py-3 bg-[#185fa5] text-white rounded-lg font-semibold hover:bg-[#134b82] transition-colors">
            Get Started
          </a>
          <a href="/cli/" class="px-6 py-3 border border-gray-300 dark:border-gray-600 rounded-lg font-semibold hover:bg-gray-50 dark:hover:bg-gray-800 transition-colors">
            CLI Reference
          </a>
        </div>

        <div class="mt-10 max-w-lg mx-auto bg-[#1e293b] rounded-lg p-4 text-left font-mono text-sm">
          <div class="text-gray-400 mb-1">$ Quick Start</div>
          <div class="text-green-400">futou</div>
          <div class="text-gray-300">install php 8.4.23</div>
          <div class="text-green-400">futou</div>
          <div class="text-gray-300">use php 8.4.23</div>
          <div class="text-green-400">futou</div>
          <div class="text-gray-300">start apache 2.4.66</div>
        </div>
      </div>
    </section>

    <section class="relative max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 pb-24">
      <div class="grid grid-cols-1 md:grid-cols-3 gap-8">
        <div class="p-6 rounded-xl border border-gray-200 dark:border-gray-700">
          <h3 class="font-semibold text-lg mb-2">Multi-Version</h3>
          <p class="text-gray-500 dark:text-gray-400 text-sm">PHP 8.2, 8.3, and 8.4 side-by-side. Switch with one command.</p>
        </div>
        <div class="p-6 rounded-xl border border-gray-200 dark:border-gray-700">
          <h3 class="font-semibold text-lg mb-2">Database Servers</h3>
          <p class="text-gray-500 dark:text-gray-400 text-sm">MariaDB and PostgreSQL with automatic data directory initialization.</p>
        </div>
        <div class="p-6 rounded-xl border border-gray-200 dark:border-gray-700">
          <h3 class="font-semibold text-lg mb-2">Web Servers</h3>
          <p class="text-gray-500 dark:text-gray-400 text-sm">Apache and Nginx with auto-generated config and per-project document roots.</p>
        </div>
      </div>
    </section>
  </div>
</template>

<script setup lang="ts">
import Ray from './Ray.vue'
</script>

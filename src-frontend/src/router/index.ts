import { createRouter, createWebHistory } from 'vue-router';

const routes = [
  { path: '/', redirect: '/home' },
  { path: '/home', name: 'home', component: () => import('@/views/HomeView.vue') },
  { path: '/favorites', name: 'favorites', component: () => import('@/views/FavoritesView.vue') },
  { path: '/history', name: 'history', component: () => import('@/views/HistoryView.vue') },
  { path: '/search', name: 'search', component: () => import('@/views/SearchView.vue') },
  { path: '/now-playing', name: 'now-playing', component: () => import('@/views/NowPlayingView.vue') },
  { path: '/playlists', name: 'playlists', component: () => import('@/views/PlaylistsView.vue') },
  { path: '/playlist/:id', name: 'playlist', component: () => import('@/views/PlaylistDetailView.vue') },
  { path: '/settings', name: 'settings', component: () => import('@/views/SettingsView.vue') },
];

export const router = createRouter({
  history: createWebHistory(),
  routes,
  scrollBehavior() {
    return { top: 0 };
  },
});

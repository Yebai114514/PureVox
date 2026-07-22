import { reactive, onMounted, onUnmounted, type Component } from 'vue';

export interface ContextMenuItem {
  label: string;
  iconComponent?: Component;
  action?: () => void;
  disabled?: boolean;
  separator?: boolean;
}

interface MenuState {
  show: boolean;
  x: number;
  y: number;
  items: ContextMenuItem[];
}

export const menuState = reactive<MenuState>({
  show: false,
  x: 0,
  y: 0,
  items: [],
});

export function showContextMenu(event: MouseEvent, items: ContextMenuItem[]) {
  event.preventDefault();
  event.stopPropagation();
  menuState.show = true;
  menuState.x = event.clientX;
  menuState.y = event.clientY;
  menuState.items = items;
}

export function hideContextMenu() {
  menuState.show = false;
  menuState.items = [];
}

export function useContextMenu() {
  function onKeyDown(e: KeyboardEvent) {
    if (!menuState.show) return;
    if (e.key === 'Escape') {
      hideContextMenu();
    }
  }

  function onScroll() {
    if (menuState.show) hideContextMenu();
  }

  function onResize() {
    if (menuState.show) hideContextMenu();
  }

  onMounted(() => {
    window.addEventListener('keydown', onKeyDown);
    window.addEventListener('scroll', onScroll, true);
    window.addEventListener('resize', onResize);
  });

  onUnmounted(() => {
    window.removeEventListener('keydown', onKeyDown);
    window.removeEventListener('scroll', onScroll, true);
    window.removeEventListener('resize', onResize);
  });

  return { showContextMenu, hideContextMenu, menuState };
}

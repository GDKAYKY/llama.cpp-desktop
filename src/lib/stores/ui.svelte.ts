class UIStore {
  isSidebarOpen = $state(true);

  toggleSidebar() {
    this.isSidebarOpen = !this.isSidebarOpen;
  }
}

export const uiStore = new UIStore();

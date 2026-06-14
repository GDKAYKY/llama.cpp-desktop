class UIStore {
  isSidebarOpen = $state(true);
  isSidebarHidden = $state(false);

  toggleSidebar() {
    this.isSidebarOpen = !this.isSidebarOpen;
  }

  hideSidebar() {
    this.isSidebarHidden = true;
    this.isSidebarOpen = false;
  }

  showSidebar() {
    this.isSidebarHidden = false;
    this.isSidebarOpen = true;
  }
}

export const uiStore = new UIStore();

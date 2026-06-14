class UIStore {
  isSidebarOpen = $state(true);
  isSidebarHidden = $state(false);
  isSessionPanelOpen = $state(false);

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

  toggleSessionPanel() {
    this.isSessionPanelOpen = !this.isSessionPanelOpen;
  }

  hideSessionPanel() {
    this.isSessionPanelOpen = false;
  }

  showSessionPanel() {
    this.isSessionPanelOpen = true;
  }
}

export const uiStore = new UIStore();

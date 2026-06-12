import { writable } from "svelte/store";

type NotificationOptions = {
  id?: string | number;
  duration?: number;
  loading?: boolean;
  progress?: number | null;
};

type NotificationState = {
  id: string | number | null;
  message: string;
  loading: boolean;
  progress: number | null;
  visible: boolean;
};

const DEFAULT_DURATION = 3000;
const DEFAULT_NOTIFICATION_ID = "app-notification";
let hideTimeout: ReturnType<typeof setTimeout> | null = null;

export const notificationState = writable<NotificationState>({
  id: null,
  message: "",
  loading: false,
  progress: null,
  visible: false,
});

function clearHideTimeout() {
  if (hideTimeout !== null) {
    clearTimeout(hideTimeout);
    hideTimeout = null;
  }
}

function show(message: string, options: NotificationOptions = {}) {
  const id = options.id ?? DEFAULT_NOTIFICATION_ID;

  clearHideTimeout();
  notificationState.set({
    id,
    message,
    loading: options.loading ?? false,
    progress: options.progress ?? null,
    visible: true,
  });

  if (!(options.loading ?? false)) {
    hideTimeout = setTimeout(() => {
      notificationState.update((current) =>
          current.id === id
          ? { ...current, visible: false, message: "", loading: false, progress: null }
          : current,
      );
      hideTimeout = null;
    }, options.duration ?? DEFAULT_DURATION);
  }

  return id;
}

export const notifications = {
  show,
  message(message: string, options?: NotificationOptions) {
    return show(message, options);
  },
  success(message: string, options?: NotificationOptions) {
    return show(message, options);
  },
  error(message: string, options?: NotificationOptions) {
    return show(message, options);
  },
  info(message: string, options?: NotificationOptions) {
    return show(message, options);
  },
  loading(message: string, options?: Omit<NotificationOptions, "loading">) {
    return show(message, {
      ...options,
      loading: true,
    });
  },
  dismiss(id?: string | number) {
    clearHideTimeout();
    notificationState.update((current) => {
      if (id !== undefined && current.id !== id) {
        return current;
      }

      return {
        ...current,
        visible: false,
        message: "",
        loading: false,
        progress: null,
      };
    });

    return id;
  },
};

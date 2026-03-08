// Stubs for icon handling - waiting for DDS renderer implementation

export function buildIconUrl(iconName: string): string {
  return '';
}

export async function fetchIconStats(): Promise<{
  initialized: boolean;
  initializing: boolean;
  statistics: {
    base_count: number;
    override_count: number;
    workshop_count: number;
    hak_count: number;
    module_count: number;
    total_count: number;
    total_size: number;
  };
  format: string;
  mimetype: string;
}> {
  return {
    initialized: true,
    initializing: false,
    statistics: {
      base_count: 0,
      override_count: 0,
      workshop_count: 0,
      hak_count: 0,
      module_count: 0,
      total_count: 0,
      total_size: 0,
    },
    format: 'dds',
    mimetype: 'image/vnd.ms-dds',
  };
}

export async function updateModuleIcons(hakList: string[]): Promise<{
  success: boolean;
  haks_loaded: number;
  statistics: Record<string, unknown>;
}> {
  return {
    success: true,
    haks_loaded: 0,
    statistics: {},
  };
}

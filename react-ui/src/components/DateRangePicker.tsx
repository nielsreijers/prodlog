import React, { useState, useRef, useEffect } from 'react';

interface DateRange {
  from: string;
  to: string;
}

interface DateRangePreset {
  label: string;
  value: DateRange;
}

interface DateRangePickerProps {
  value: DateRange;
  onChange: (range: DateRange) => void;
}

const DateRangePicker: React.FC<DateRangePickerProps> = ({ value, onChange }) => {
  const [isOpen, setIsOpen] = useState(false);
  const [tempRange, setTempRange] = useState<DateRange>(value);
  const dropdownRef = useRef<HTMLDivElement>(null);

  // Close dropdown when clicking outside
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (dropdownRef.current && !dropdownRef.current.contains(event.target as Node)) {
        setIsOpen(false);
      }
    };

    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, []);

  // Generate date presets
  const generatePresets = (): DateRangePreset[] => {
    const today = new Date();
    const yesterday = new Date(today);
    yesterday.setDate(yesterday.getDate() - 1);
    
    const last7Days = new Date(today);
    last7Days.setDate(last7Days.getDate() - 7);
    
    const last30Days = new Date(today);
    last30Days.setDate(last30Days.getDate() - 30);
    
    const last365Days = new Date(today);
    last365Days.setDate(last365Days.getDate() - 365);

    const formatDate = (date: Date) => date.toISOString().split('T')[0];

    return [
      {
        label: 'Today',
        value: {
          from: formatDate(today),
          to: formatDate(today)
        }
      },
      {
        label: 'Yesterday',
        value: {
          from: formatDate(yesterday),
          to: formatDate(yesterday)
        }
      },
      {
        label: 'Last 7 days',
        value: {
          from: formatDate(last7Days),
          to: formatDate(today)
        }
      },
      {
        label: 'Last 30 days',
        value: {
          from: formatDate(last30Days),
          to: formatDate(today)
        }
      },
      {
        label: 'Last 365 days',
        value: {
          from: formatDate(last365Days),
          to: formatDate(today)
        }
      },
      {
        label: 'All time',
        value: {
          from: '',
          to: ''
        }
      }
    ];
  };

  const presets = generatePresets();

  const handlePresetClick = (preset: DateRangePreset) => {
    onChange(preset.value);
    setIsOpen(false);
  };

  const handleCustomApply = () => {
    onChange(tempRange);
    setIsOpen(false);
  };

  const handleCustomCancel = () => {
    setTempRange(value);
    setIsOpen(false);
  };

  // Format display text
  const getDisplayText = () => {
    if (!value.from && !value.to) {
      return 'All time';
    }
    
    // Check if matches any preset
    const matchingPreset = presets.find(
      preset => preset.value.from === value.from && preset.value.to === value.to
    );
    
    if (matchingPreset) {
      return matchingPreset.label;
    }
    
    // Custom range
    if (value.from && value.to) {
      if (value.from === value.to) {
        return value.from;
      }
      return `${value.from} to ${value.to}`;
    }
    
    if (value.from) {
      return `From ${value.from}`;
    }
    
    if (value.to) {
      return `Until ${value.to}`;
    }
    
    return 'Select date range';
  };

  return (
    <div className="date-range-picker" ref={dropdownRef}>
      <button 
        type="button"
        className="date-range-button"
        onClick={() => setIsOpen(!isOpen)}
      >
        <span className="date-range-text">{getDisplayText()}</span>
        <span className="date-range-arrow">▼</span>
      </button>

      {isOpen && (
        <div className="date-range-dropdown">
          <div className="date-range-presets">
            <div className="date-range-section-title">Quick select</div>
            {presets.map((preset) => (
              <button
                key={preset.label}
                type="button"
                className="date-range-preset"
                onClick={() => handlePresetClick(preset)}
              >
                {preset.label}
              </button>
            ))}
          </div>

          <div className="date-range-custom">
            <div className="date-range-section-title">Custom range</div>
            <div className="date-range-inputs">
              <input
                type="date"
                value={tempRange.from}
                onChange={(e) => setTempRange({ ...tempRange, from: e.target.value })}
                placeholder="From date"
              />
              <span className="date-range-separator">to</span>
              <input
                type="date"
                value={tempRange.to}
                onChange={(e) => setTempRange({ ...tempRange, to: e.target.value })}
                placeholder="To date"
              />
            </div>
            <div className="date-range-actions">
              <button 
                type="button" 
                className="date-range-cancel"
                onClick={handleCustomCancel}
              >
                Cancel
              </button>
              <button 
                type="button" 
                className="date-range-apply"
                onClick={handleCustomApply}
              >
                Apply
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

export default DateRangePicker; 
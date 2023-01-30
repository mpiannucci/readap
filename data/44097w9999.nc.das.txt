Attributes {
    time {
        String long_name "Epoch Time";
        String short_name "time";
        String standard_name "time";
        String units "seconds since 1970-01-01 00:00:00 UTC";
    }
    frequency {
        String long_name "Frequency";
        String short_name "frequency";
        String standard_name "frequency";
        String units "Hz";
    }
    latitude {
        String long_name "Latitude";
        String short_name "latitude";
        String standard_name "latitude";
        String units "degrees_north";
    }
    longitude {
        String long_name "Longitude";
        String short_name "longitude";
        String standard_name "longitude";
        String units "degrees_east";
    }
    spectral_wave_density {
        String long_name "Spectral Wave Density";
        String short_name "swden";
        String standard_name "spectral_wave_density";
        String units "(meter * meter)/Hz";
        Float32 _FillValue 999.0;
    }
    mean_wave_dir {
        String long_name "Mean Wave Direction";
        String short_name "alpha1";
        String standard_name "mean_wave_direction";
        String units "degrees_true";
        Int32 _FillValue 999;
    }
    principal_wave_dir {
        String long_name "Principal Wave Direction";
        String short_name "alpha2";
        String standard_name "principal_wave_direction";
        String units "degrees_true";
        Int32 _FillValue 999;
    }
    wave_spectrum_r1 {
        String long_name "First Normalized Polar Coordinate Derived from the Fourier Coefficients";
        String short_name "r1";
        String standard_name "wave_spectrum_r1";
        Float32 _FillValue 999.0;
    }
    wave_spectrum_r2 {
        String long_name "Second Normalized Polar Coordinate Derived from the Fourier Coefficients";
        String short_name "r2";
        String standard_name "wave_spectrum_r2";
        Float32 _FillValue 999.0;
    }
    NC_GLOBAL {
        String institution "NOAA National Data Buoy Center and Participators in Data Assembly Center";
        String url "http://dods.ndbc.noaa.gov";
        String quality "Automated QC checks with daily manual QC";
        String conventions "COARDS";
        String station "44097";
        String comment "Block Island, RI  (154)";
        String location "40.969 N 71.127 W ";
    }
    DODS_EXTRA {
        String Unlimited_Dimension "time";
    }
}
